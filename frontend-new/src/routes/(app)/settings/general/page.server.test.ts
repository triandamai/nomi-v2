import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const answers: Record<string, unknown> = {
	'/api/profile': { email: 'sari@example.com' },
	'/api/auth/google': { configured: true, password: true, google_email: null },
	'/api/devices': [
		{ id: 'here', current: true },
		{ id: 'phone', current: false },
		{ id: 'tablet', current: false },
	],
	'/api/preferences': { theme: 'dark', timezone: 'Asia/Jakarta', language: 'en', has_stored_timezone: true },
	'/api/notifications/preferences': { email_account: true, email_promos: false },
};
const backend = (answer?: Parameters<typeof fakeBackend>[0]) =>
	fakeBackend(answer ?? ((call) => (call.method === 'GET' ? json(answers[call.url.replace('http://api.test', '')]) : new Response(null, { status: 204 }))));
const run = (name: keyof typeof actions, form: Record<string, string>, answer?: Parameters<typeof fakeBackend>[0]) => {
	const api = backend(answer);
	return { api, result: actions[name](fakeEvent({ url: 'http://localhost/settings/general', form, fetch: api.fetch, locals: { accessToken: 'token' } }) as never) };
};

describe('Settings › General', () => {
	it('names itself, says changes save as they’re made, and loads what it shows', async () => {
		const api = backend();
		const data = await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ title: 'General', autosave: true, email: 'sari@example.com', methods: { password: true } });
	});

	it('shows the error page when it can’t load', async () => {
		const api = backend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});

	it('sends the person to Google to add their account', async () => {
		const { result } = run('linkGoogle', {}, () => json({ url: 'https://accounts.google.com/o?x=1' }));
		expect(await thrown(() => result)).toMatchObject({ status: 303, location: 'https://accounts.google.com/o?x=1' });
	});

	it('removes Google, or says why it can’t', async () => {
		expect(await run('unlinkGoogle', {}).result).toEqual({ unlinked: true });
		const only = run('unlinkGoogle', {}, () => new Response('Google is the only way you sign in. Set a password first.', { status: 409 }));
		expect(await only.result).toMatchObject({ status: 409, data: { message: 'Google is the only way you sign in. Set a password first.' } });
	});

	it('signs out every device but this one', async () => {
		const { api, result } = run('signOutOthers', {});
		expect(await result).toEqual({ signedOut: 2 });
		expect(api.calls.filter((call) => call.method === 'DELETE').map((call) => call.url)).toEqual(['http://api.test/api/devices/phone', 'http://api.test/api/devices/tablet']);
	});

	it('saves which emails they get', async () => {
		const { api, result } = run('emails', { email_account: 'false', email_promos: 'true' }, () => json({ email_account: false, email_promos: true }));
		expect(await result).toEqual({ emails: { email_account: false, email_promos: true } });
		expect(api.calls[0].body).toEqual({ email_account: false, email_promos: true });
	});
});
