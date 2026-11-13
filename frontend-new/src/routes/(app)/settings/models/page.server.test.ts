import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const options = {
	groups: [
		{ id: 'nomi', source: 'nomi', provider_id: 'nomi', has_logo: false, label: 'Nomi', connection_id: null, error: null, models: [{ key: 'admin:a1', label: 'deepseek-flash', model_id: 'deepseek-flash', free: true, enabled: true }] },
	],
	current: null,
};
const run = (name: keyof typeof actions, form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0] = () => new Response(null, { status: 204 })) => {
	const api = fakeBackend(answer);
	return { api, result: actions[name](fakeEvent({ url: 'http://localhost/settings/models', form, fetch: api.fetch, locals: { accessToken: 'token' } }) as never) };
};

describe('Settings › Models & providers', () => {
	it('names itself with how many providers and models are in the picker', async () => {
		const api = fakeBackend((call) => (call.url.endsWith('/model-options') ? json(options) : new Response('', { status: 500 })));
		const data = await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ title: 'Models & providers', meta: '1 provider · 1 model in picker' });
	});

	it('shows the error page without the picker’s models', async () => {
		const api = fakeBackend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});

	it('sets the model new chats start with, and Koda’s', async () => {
		const chats = run('setDefault', { key: 'conn:c1:claude-sonnet-5-5' });
		expect(await chats.result).toEqual({ saved: 'default' });
		expect(chats.api.calls[0].body).toEqual({ key: 'conn:c1:claude-sonnet-5-5' });
		const koda = run('setCoding', { value: 'admin:a1' });
		expect(await koda.result).toEqual({ saved: 'coding' });
		expect(koda.api.calls[0].body).toEqual({ kind: 'admin', admin_model_id: 'a1' });
		const same = run('setCoding', { value: 'same_as_chat' });
		await same.result;
		expect(same.api.calls[0].body).toEqual({ kind: 'same_as_chat' });
		const odd = run('setCoding', { value: 'conn:c1:x' });
		expect(await odd.result).toMatchObject({ status: 400 });
	});

	it('shows and hides models in the picker', async () => {
		const { api, result } = run('setShown', { changes: '[{"key":"admin:a2","enabled":false}]' });
		expect(await result).toEqual({ saved: 'shown' });
		expect(api.calls[0].body).toEqual({ changes: [{ key: 'admin:a2', enabled: false }] });
		expect(await run('setShown', { changes: 'nope' }).result).toMatchObject({ status: 400 });
	});

	it('connects with a key the provider accepts, and lists what it offers', async () => {
		const { api, result } = run('connect', { provider_id: 'anthropic', api_key: ' sk-ant ' }, (call) =>
			call.url.endsWith('/models') ? json({ models: [{ id: 'claude-sonnet-5-5', label: 'Claude Sonnet 5.5' }] }) : json({ id: 'c1', label: 'Anthropic' }, 201),
		);
		expect(await result).toEqual({ connection: { id: 'c1', label: 'Anthropic' }, models: [{ id: 'claude-sonnet-5-5', label: 'Claude Sonnet 5.5' }] });
		expect(api.calls[0].body).toEqual({ provider_id: 'anthropic', api_key: 'sk-ant' });
	});

	it('says plainly when the provider refuses the key or can’t be reached', async () => {
		const refused = run('connect', { provider_id: 'anthropic', api_key: 'bad' }, () => json({ error: 'key_rejected', detail: 'invalid x-api-key' }, 400));
		expect(await refused.result).toMatchObject({ status: 400, data: { field: 'key', message: 'The provider refused that key: invalid x-api-key' } });
		const away = run('connect', { provider_id: 'ollama', base_url: 'http://localhost:11434/v1' }, () => json({ error: 'unreachable', detail: 'connection refused' }, 502));
		expect(await away.result).toMatchObject({ status: 502, data: { field: 'key', message: 'Couldn’t reach it: connection refused' } });
		expect(await run('connect', { provider_id: '' }).result).toMatchObject({ status: 400 });
	});

	it('puts the chosen models of a new connection in the picker', async () => {
		const { api, result } = run('showModels', { connection_id: 'c1', models: '["claude-sonnet-5-5","claude-haiku-5-5"]' });
		expect(await result).toEqual({ saved: 'shown' });
		expect(api.calls[0].body).toEqual({ changes: [{ key: 'conn:c1:claude-sonnet-5-5', enabled: true }, { key: 'conn:c1:claude-haiku-5-5', enabled: true }] });
	});

	it('disconnects a provider', async () => {
		const { api, result } = run('disconnect', { id: 'c1' });
		expect(await result).toEqual({ disconnected: 'c1' });
		expect(api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/llm/connections/c1' });
	});
});
