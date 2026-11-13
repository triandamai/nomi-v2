import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const connections = await import('./connections');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('Connections on the server', () => {
	it('loads what’s connected, each part on its own', async () => {
		const { event } = eventWith((call) => (call.url.endsWith('/github') ? new Response('', { status: 500 }) : json({ google: { configured: true }, hevy: null })));
		expect(await connections.loadConnections(event)).toEqual({ overview: { google: { configured: true }, hevy: null }, github: null });
	});

	it('starts Google with the services asked for and the chat to come back to', async () => {
		const { backend, event } = eventWith(() => json({ url: 'https://accounts.google.com/o' }));
		expect(await connections.startGoogle(event, ['gmail', 'calendar'], 's1')).toEqual({ ok: true, data: { url: 'https://accounts.google.com/o' } });
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/connections/google/start', body: { services: ['gmail', 'calendar'], resume_session_id: 's1' } });
	});

	it('passes on a refused Hevy key with its reason', async () => {
		const { event } = eventWith(() => json({ error: 'key_rejected', detail: 'Hevy said the key is invalid' }, 400));
		expect(await connections.connectHevy(event, 'bad')).toEqual({ ok: false, status: 400, code: 'key_rejected', message: 'Hevy said the key is invalid' });
		const quiet = eventWith(() => json({ error: 'key_rejected', detail: null }, 400));
		expect(await connections.connectHevy(quiet.event, 'bad')).toEqual({ ok: false, status: 400, code: 'key_rejected', message: '' });
	});

	it('passes on GitHub’s plain-text refusal', async () => {
		const { event } = eventWith(() => new Response("GitHub didn't accept the token; connect GitHub again", { status: 400 }));
		expect(await connections.githubToken(event, 'ghp_x')).toEqual({ ok: false, status: 400, message: "GitHub didn't accept the token; connect GitHub again" });
	});

	it('finishes and disconnects', async () => {
		const { backend, event } = eventWith((call) => (call.method === 'DELETE' ? new Response(null, { status: 204 }) : json({ email: 'a@b.c', services: ['gmail'], resume_session_id: null })));
		await connections.finishGoogle(event, 'c', 's');
		await connections.disconnectGoogle(event);
		await connections.disconnectGithub(event);
		await connections.disconnectHevy(event);
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', '')])).toEqual([
			['POST', '/api/connections/google/callback'],
			['DELETE', '/api/connections/google'],
			['DELETE', '/api/connections/github'],
			['DELETE', '/api/connections/hevy'],
		]);
	});
});
