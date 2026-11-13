import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const models = await import('./models');

const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('Models & providers on the server', () => {
	it('loads the picker’s models, Nomi’s and Koda’s, the connections, this month and the catalog', async () => {
		const answers: Record<string, unknown> = {
			'/api/llm/model-options': { groups: [], current: null },
			'/api/llm/models': { admin_models: [], coding: { kind: 'default' }, default_coding_model_id: null },
			'/api/llm/connections': { connections: [{ id: 'c1' }] },
			'/api/usage': { month: '2026-10' },
			'/api/llm/providers': { providers: [{ id: 'anthropic' }] },
		};
		const { event } = eventWith((call) => json(answers[call.url.replace('http://api.test', '')]));
		expect(await models.loadModels(event)).toEqual({
			options: { groups: [], current: null },
			models: { admin_models: [], coding: { kind: 'default' }, default_coding_model_id: null },
			connections: [{ id: 'c1' }],
			usage: { month: '2026-10' },
			catalog: [{ id: 'anthropic' }],
		});
	});

	it('says it couldn’t load without the picker’s models, and leaves out the rest when they fail', async () => {
		const none = eventWith(() => new Response('', { status: 500 }));
		expect(await models.loadModels(none.event)).toBeNull();
		const some = eventWith((call) => (call.url.endsWith('/api/llm/model-options') ? json({ groups: [], current: null }) : new Response('', { status: 500 })));
		expect(await models.loadModels(some.event)).toEqual({ options: { groups: [], current: null }, models: null, connections: [], usage: null, catalog: [] });
	});

	it('sets the defaults and what’s shown', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		await models.setDefaultModel(event, 'admin:a1');
		await models.setCodingModel(event, { kind: 'same_as_chat' });
		await models.setShown(event, [{ key: 'admin:a2', enabled: true }]);
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body])).toEqual([
			['PUT', '/api/llm/default-model', { key: 'admin:a1' }],
			['PUT', '/api/llm/coding-selection', { kind: 'same_as_chat' }],
			['PUT', '/api/llm/model-options', { changes: [{ key: 'admin:a2', enabled: true }] }],
		]);
	});

	it('passes on why the provider refused the key', async () => {
		const { event } = eventWith(() => json({ error: 'key_rejected', detail: 'invalid x-api-key' }, 400));
		expect(await models.connectProvider(event, { provider_id: 'anthropic', api_key: 'bad' })).toMatchObject({ ok: false, status: 400, code: 'key_rejected' });
	});

	it('connects, lists the connection’s models, and disconnects', async () => {
		const { backend, event } = eventWith((call) => (call.method === 'DELETE' ? new Response(null, { status: 204 }) : call.url.endsWith('/models') ? json({ models: [{ id: 'm1', label: null }] }) : json({ id: 'c1' }, 201)));
		expect(await models.connectProvider(event, { provider_id: 'anthropic', api_key: 'sk' })).toEqual({ ok: true, data: { id: 'c1' } });
		expect(await models.connectionModels(event, 'c1')).toEqual({ ok: true, data: { models: [{ id: 'm1', label: null }] } });
		expect(await models.disconnectProvider(event, 'c1')).toEqual({ ok: true, data: undefined });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', '')])).toEqual([
			['POST', '/api/llm/connections'],
			['POST', '/api/llm/connections/c1/models'],
			['DELETE', '/api/llm/connections/c1'],
		]);
	});
});
