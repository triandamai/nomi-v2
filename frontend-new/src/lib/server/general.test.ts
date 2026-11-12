import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const general = await import('./general');

function eventWith(answer: Parameters<typeof fakeBackend>[0]) {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
}

const answers: Record<string, unknown> = {
	'/api/profile': { email: 'sari@example.com' },
	'/api/auth/google': { configured: true, password: true, google_email: null },
	'/api/devices': [{ id: 'd1', current: true }],
	'/api/preferences': { theme: 'dark', timezone: 'Asia/Jakarta', language: 'id', has_stored_timezone: true },
	'/api/notifications/preferences': { email_account: true, email_promos: false },
};

describe('Settings › General on the server', () => {
	it('loads the email, how they sign in, their devices, preferences and emails', async () => {
		const { event } = eventWith((call) => json(answers[call.url.replace('http://api.test', '')]));
		expect(await general.loadGeneral(event)).toEqual({
			email: 'sari@example.com',
			methods: { configured: true, password: true, google_email: null },
			devices: [{ id: 'd1', current: true }],
			preferences: { theme: 'dark', timezone: 'Asia/Jakarta', language: 'id', has_stored_timezone: true },
			emails: { email_account: true, email_promos: false },
		});
	});

	it('still loads when the devices or the emails can’t, without them', async () => {
		const { event } = eventWith((call) => (/devices|notifications/.test(call.url) ? new Response('', { status: 500 }) : json(answers[call.url.replace('http://api.test', '')])));
		expect(await general.loadGeneral(event)).toMatchObject({ devices: null, emails: null });
	});

	it('says it couldn’t load without the profile, the sign-in methods or the preferences', async () => {
		const { event } = eventWith((call) => (call.url.endsWith('/api/auth/google') ? new Response('', { status: 500 }) : json(answers[call.url.replace('http://api.test', '')])));
		expect(await general.loadGeneral(event)).toBeNull();
	});

	it('starts adding Google, and removes it', async () => {
		const { backend, event } = eventWith((call) => (call.method === 'POST' ? json({ url: 'https://accounts.google.com/o?x=1' }) : new Response(null, { status: 204 })));
		expect(await general.startGoogleLink(event)).toEqual({ ok: true, data: { url: 'https://accounts.google.com/o?x=1' } });
		expect(await general.unlinkGoogle(event)).toEqual({ ok: true, data: undefined });
		expect(backend.calls.map((call) => [call.method, call.url])).toEqual([
			['POST', 'http://api.test/api/auth/google/link'],
			['DELETE', 'http://api.test/api/auth/google'],
		]);
	});

	it('passes on why Google can’t be removed', async () => {
		const { event } = eventWith(() => new Response('Google is the only way you sign in. Set a password first.', { status: 409 }));
		expect(await general.unlinkGoogle(event)).toEqual({ ok: false, status: 409, message: 'Google is the only way you sign in. Set a password first.' });
	});

	it('signs the other devices out, one by one', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		expect(await general.signOutDevices(event, ['d2', 'd3'])).toEqual({ ok: true, data: undefined });
		expect(backend.calls.map((call) => [call.method, call.url])).toEqual([
			['DELETE', 'http://api.test/api/devices/d2'],
			['DELETE', 'http://api.test/api/devices/d3'],
		]);
	});
});
