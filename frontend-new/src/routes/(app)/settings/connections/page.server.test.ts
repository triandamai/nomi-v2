import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const google = await import('../../../connections/google/callback/+server');
const github = await import('../../../connections/github/callback/+server');

const CHAT = '0b9a6f1e-2c3d-4e5f-8a9b-0c1d2e3f4a5b';
const run = (name: keyof typeof actions, form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0] = () => new Response(null, { status: 204 })) => {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url: 'http://localhost/settings/connections', form, fetch: api.fetch, locals: { accessToken: 'token' } });
	return { api, event, result: actions[name](event as never) };
};

describe('Settings › Connections', () => {
	it('names itself with how many are connected', async () => {
		const api = fakeBackend((call) => (call.url.endsWith('/github') ? json({ oauth: true, connection: { login: 'mario', kind: 'oauth' } }) : json({ google: { configured: true, email: null, services: [] }, hevy: null })));
		const data = await load(fakeEvent({ url: `http://localhost/settings/connections?resume=${CHAT}`, fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ title: 'Connections', meta: '1 connected', resume: CHAT });
	});

	it('shows the error page when nothing can be read', async () => {
		const api = fakeBackend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ url: 'http://localhost/settings/connections', fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});

	it('sends the person to Google with the services they chose, and the chat to come back to', async () => {
		const api = fakeBackend(() => json({ url: 'https://accounts.google.com/o' }));
		const body = new FormData();
		body.append('service', 'gmail');
		body.append('service', 'calendar');
		body.append('service', 'nonsense');
		body.append('resume', CHAT);
		const event = { ...fakeEvent({ url: 'http://localhost/settings/connections', fetch: api.fetch, locals: { accessToken: 'token' } }), request: new Request('http://localhost/settings/connections', { method: 'POST', body }) };
		expect(await thrown(() => actions.google(event as never))).toMatchObject({ status: 303, location: 'https://accounts.google.com/o' });
		// Google comes back to this app.
		expect(api.calls[0].body).toEqual({ services: ['gmail', 'calendar'], resume_session_id: CHAT, origin: 'http://localhost' });
	});

	it('asks for at least one Google service', async () => {
		const { api, result } = run('google', {});
		expect(await result).toMatchObject({ status: 400, data: { app: 'google' } });
		expect(api.calls).toEqual([]);
	});

	it('sends the person to GitHub, keeping the chat to come back to', async () => {
		const { event, result } = run('github', { resume: CHAT }, () => json({ url: 'https://github.com/login/oauth' }));
		expect(await thrown(() => result)).toMatchObject({ status: 303, location: 'https://github.com/login/oauth' });
		expect(event.jar.get('nm_github_back')).toBe(`/chat/${CHAT}`);
	});

	it('connects with a GitHub token or a Hevy key, and says when one is refused', async () => {
		expect(await run('githubToken', { token: 'ghp_x' }, () => json({ oauth: false, connection: { login: 'm' } })).result).toEqual({ connected: 'github' });
		const refused = run('hevy', { api_key: 'bad' }, () => json({ error: 'key_rejected', detail: null }, 400));
		expect(await refused.result).toMatchObject({ status: 400, data: { app: 'hevy', message: 'Hevy didn’t accept that key. Copy it again from Hevy’s settings and paste it here.' } });
		expect(await run('hevy', { api_key: ' ' }).result).toMatchObject({ status: 400 });
	});

	it('disconnects each app, and nothing else', async () => {
		const { api, result } = run('disconnect', { app: 'hevy' });
		expect(await result).toEqual({ disconnected: 'hevy' });
		expect(api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/connections/hevy' });
		expect(await run('disconnect', { app: 'slack' }).result).toMatchObject({ status: 400 });
	});
});

describe('coming back from Google and GitHub', () => {
	const back = (handler: (event: never) => unknown, query: string, answer: Parameters<typeof fakeBackend>[0], cookies: Record<string, string> = {}) => {
		const api = fakeBackend(answer);
		return handler(fakeEvent({ url: `http://localhost/x${query}`, fetch: api.fetch, cookies, locals: { accessToken: 'token' } }) as never);
	};

	it('finishes Google, back to the chat that asked or to Connections', async () => {
		expect(await thrown(() => back(google.GET, '?code=c&state=s', () => json({ email: 'a@b.c', services: ['gmail'], resume_session_id: CHAT })))).toMatchObject({ location: `/chat/${CHAT}` });
		expect(await thrown(() => back(google.GET, '?code=c&state=s', () => json({ email: 'a@b.c', services: ['gmail'], resume_session_id: null })))).toMatchObject({ location: '/settings/connections?connected=google' });
	});

	it('says so when the person declined or it failed', async () => {
		expect(await thrown(() => back(google.GET, '?error=access_denied', () => json({})))).toMatchObject({ location: '/settings/connections?error=declined' });
		expect(await thrown(() => back(google.GET, '?code=c&state=s', () => new Response('bad state', { status: 400 })))).toMatchObject({ location: '/settings/connections?error=failed' });
	});

	it('finishes GitHub, back to where the person was', async () => {
		expect(await thrown(() => back(github.GET, '?code=c&state=s', () => json({ oauth: true, connection: {} }), { nm_github_back: `/chat/${CHAT}` }))).toMatchObject({ location: `/chat/${CHAT}` });
		expect(await thrown(() => back(github.GET, '?code=c&state=s', () => json({ oauth: true, connection: {} })))).toMatchObject({ location: '/settings/connections?connected=github' });
	});
});
