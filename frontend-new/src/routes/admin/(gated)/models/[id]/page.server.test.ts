import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const model = { id: 'm1', label: 'DeepSeek Flash', connection_id: 'c1' };
const answer = (call: BackendCall) =>
	call.url.endsWith('/llm/models') ? json([model]) : call.url.endsWith('/llm/connections') ? json({ connections: [{ id: 'c1' }] }) : call.url.endsWith('/llm/providers') ? json({ providers: [{ id: 'openai' }] }) : json({});

function event(id: string, form?: Record<string, string>, reply: (call: BackendCall) => Response = answer) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ params: { id }, form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('a model', () => {
	it('opens new or from the list, with the connected providers and the catalog', async () => {
		expect(await load(event('new').event)).toMatchObject({ model: null, connections: [{ id: 'c1' }], providers: [{ id: 'openai' }] });
		expect(await load(event('m1').event)).toMatchObject({ title: 'DeepSeek Flash', model: { ok: true, data: model } });
		await expect(load(event('nope').event)).rejects.toMatchObject({ status: 404 });
	});

	it('adds a model and goes back to the list; saves one in place', async () => {
		const fields = { label: 'Flash', source: 'connection', connection_id: 'c1', model_id: 'deepseek-flash', media_mode: 'auto' };
		const adding = event('new', fields, () => json({ id: 'm9' }, 201));
		expect(await thrown(() => actions.save(adding.event))).toMatchObject({ status: 303, location: '/admin/models' });
		expect(adding.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/settings/llm/models' });

		const saving = event('m1', fields, () => json(model));
		expect(await actions.save(saving.event)).toMatchObject({ action: 'save', ok: true });
		expect(saving.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/settings/llm/models/m1' });
	});

	it('finds what a provider serves: a connected one by its key, or a key typed here', async () => {
		const connected = event('new', { source: 'connection', connection_id: 'c1' }, () => json({ models: [{ id: 'gpt-x', label: null }] }));
		expect(await actions.findModels(connected.event)).toMatchObject({ action: 'findModels', ok: true, data: { models: [{ id: 'gpt-x', label: null }] } });
		expect(connected.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/settings/llm/connections/c1/models' });

		const own = event('m1', { source: 'own', provider: 'openai', api_key: '', base_url: '' }, () => json({ models: [] }));
		await actions.findModels(own.event);
		expect(own.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/settings/llm/models/fetch-models', body: { provider: 'openai', api_key: null, base_url: null, existing_model_id: 'm1' } });
	});

	it('deletes a model and goes back to the list', async () => {
		const deleting = event('m1', {}, () => new Response(null, { status: 204 }));
		expect(await thrown(() => actions.delete(deleting.event))).toMatchObject({ status: 303, location: '/admin/models' });
		expect(deleting.api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/admin/settings/llm/models/m1' });
	});
});
