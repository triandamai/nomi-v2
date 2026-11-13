import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const nomi = { agent_type: 'chitchat', name: 'Nomi', role: 'Talks with you', is_dynamic: false, state: 'idle', status: '' };

describe('Settings › Crew', () => {
	it('names itself and says how many are on the crew', async () => {
		const api = fakeBackend((call) => (call.url.endsWith('/api/agents') ? json({ agents: [nomi, { ...nomi, agent_type: 'money', name: 'Dana', state: 'working' }] }) : new Response('', { status: 500 })));
		const data = await load(fakeEvent({ url: 'http://localhost/settings/crew', fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ title: 'Crew', meta: '2 members · 1 working', personality: null });
	});

	it('shows the error page when the crew can’t be read', async () => {
		const api = fakeBackend(() => new Response('', { status: 502 }));
		expect(await thrown(() => load(fakeEvent({ url: 'http://localhost/settings/crew', fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});

	it('restores a version of the personality, or says it’s gone', async () => {
		const restore = (version: string, answer: Parameters<typeof fakeBackend>[0]) => {
			const api = fakeBackend(answer);
			return { api, result: actions.restore(fakeEvent({ url: 'http://localhost/settings/crew', form: { version }, fetch: api.fetch, locals: { accessToken: 'token' } }) as never) };
		};
		const done = restore('2', () => new Response(null, { status: 204 }));
		expect(await done.result).toEqual({ restored: 2 });
		expect(done.api.calls[0].body).toEqual({ version: 2 });
		expect(await restore('9', () => new Response('version not found', { status: 404 })).result).toMatchObject({ status: 404, data: { message: 'That version isn’t there any more.' } });
		const bad = restore('x', () => new Response(null, { status: 204 }));
		expect(await bad.result).toMatchObject({ status: 400 });
		expect(bad.api.calls).toEqual([]);
	});
});
