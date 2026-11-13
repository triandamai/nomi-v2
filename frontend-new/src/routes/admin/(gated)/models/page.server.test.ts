import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const run = (form: Record<string, string>, answer: (call: BackendCall) => Response = () => new Response(null, { status: 204 })) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
};
const sent = (api: { calls: BackendCall[] }) => api.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body]);

describe('models', () => {
	it('lists the models', async () => {
		const api = fakeBackend(() => json([{ id: 'm1' }]));
		expect(await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toEqual({ models: { ok: true, data: [{ id: 'm1' }] } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/settings/llm/models');
	});

	it('turns a model on or off', async () => {
		const { api, event } = run({ id: 'm1', enabled: '' });
		expect(await actions.enable(event)).toMatchObject({ action: 'enable', ok: true });
		expect(sent(api)).toEqual([['PUT', '/api/admin/settings/llm/models/m1/enabled', { enabled: false }]]);
	});

	it('changes only the roles that changed: the default, the files model and the coding model', async () => {
		const changed = run({ default_id: 'm2', was_default: 'm1', files_id: '', was_files: 'm3', coding_id: 'm1', was_coding: 'm1' });
		expect(await actions.roles(changed.event)).toMatchObject({ action: 'roles', ok: true });
		expect(sent(changed.api)).toEqual([
			['PUT', '/api/admin/settings/llm/models/m2/default', undefined],
			['PUT', '/api/admin/settings/llm/files-model', { id: null }],
		]);
	});

	it('stops at the first role the backend refuses, with its reason', async () => {
		const { api, event } = run({ default_id: 'm2', was_default: 'm1', files_id: 'm4', was_files: '', coding_id: '', was_coding: '' }, () => json({ error: 'that model is switched off' }, 400));
		expect(await actions.roles(event)).toMatchObject({ status: 400, data: { action: 'roles', problem: { message: 'that model is switched off' } } });
		expect(api.calls).toHaveLength(1);
	});
});
