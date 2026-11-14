import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const start = await import('./+server');
const callback = await import('./callback/+server');

describe('continuing with Google', () => {
	it('goes to Google, remembering where to come back to', async () => {
		const api = fakeBackend(() => json({ url: 'https://accounts.google.com/o/oauth2/auth?state=s1' }));
		const event = fakeEvent({ url: 'http://localhost/auth/google', form: { redirect_to: '/chat/7' }, fetch: api.fetch });
		expect(await thrown(() => start.POST(event as never))).toMatchObject({ status: 303, location: 'https://accounts.google.com/o/oauth2/auth?state=s1' });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/google/start', method: 'POST', body: { invite_code: null, origin: 'http://localhost' } });
		expect(event.jar.get('google_return_to')).toBe('/chat/7');
		expect(event.cookieOptions.get('google_return_to')).toMatchObject({ httpOnly: true, maxAge: 600 });
	});

	it('brings the invite along, so a new account joins its workspace', async () => {
		const api = fakeBackend(() => json({ url: 'https://accounts.google.com/x' }));
		await thrown(() => start.POST(fakeEvent({ url: 'http://localhost/auth/google', form: { redirect_to: '/invite/Xk29' }, fetch: api.fetch }) as never));
		expect(api.calls[0].body).toEqual({ invite_code: 'Xk29', origin: 'http://localhost' });
	});

	it('asks Google to come back to this app, wherever the server’s redirect address points', async () => {
		const api = fakeBackend(() => json({ url: 'https://accounts.google.com/x' }));
		await thrown(() => start.POST(fakeEvent({ url: 'https://new.nomi.test/auth/google', form: {}, fetch: api.fetch }) as never));
		expect(api.calls[0].body).toMatchObject({ origin: 'https://new.nomi.test' });
	});

	it('comes back to sign in saying Google is unavailable when the server has no Google', async () => {
		const api = fakeBackend(() => new Response('Google sign-in isn’t set up on this server yet.', { status: 503 }));
		const event = fakeEvent({ url: 'http://localhost/auth/google', form: { redirect_to: '/chat/7' }, fetch: api.fetch });
		expect(await thrown(() => start.POST(event as never))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Fchat%2F7&google_error=unavailable' });
	});
});

describe('back from Google', () => {
	const back = (query: string, answer: Parameters<typeof fakeBackend>[0], options: { cookies?: Record<string, string>; locals?: Partial<App.Locals> } = {}) => {
		const api = fakeBackend(answer);
		const event = fakeEvent({ url: `http://localhost/auth/google/callback${query}`, fetch: api.fetch, cookies: { google_return_to: '/chat/7', ...options.cookies }, locals: options.locals });
		return { api, event, run: () => callback.GET(event as never) };
	};

	it('signs an existing account in and goes back where it was going, Google marked as last used', async () => {
		const { api, event, run } = back('?code=c&state=s', (call) => (call.url.endsWith('/callback') ? json({ email: 'ana@gmail.com', is_new: false, linked: false, access_token: 'a1', refresh_token: 'r1' }) : json({})));
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/chat/7' });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/google/callback', body: { code: 'c', state: 's' } });
		expect(event.jar.get('access_token')).toBe('a1');
		expect(event.jar.get('nm_last_sign_in')).toBe('google');
		expect(event.jar.has('google_return_to')).toBe(false);
	});

	it('takes a brand-new account to its first run', async () => {
		const { run } = back('?code=c&state=s', (call) => (call.url.endsWith('/callback') ? json({ email: 'new@gmail.com', is_new: true, linked: false, access_token: 'a1', refresh_token: 'r1' }) : new Response(null, { status: 204 })));
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/first-run?redirect_to=%2Fchat%2F7' });
	});

	it('says the person cancelled when Google sends them back without a code', async () => {
		for (const query of ['?error=access_denied&state=s', '?state=s', '']) {
			const { api, run } = back(query, () => json({}));
			expect(await thrown(run)).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Fchat%2F7&google_error=cancelled' });
			expect(api.calls).toEqual([]);
		}
	});

	it('names what went wrong: another account, an unverified email, an expired attempt', async () => {
		expect(await thrown(back('?code=c&state=s', () => new Response('That Google account already signs in to a different Nomi account.', { status: 409 })).run)).toMatchObject({
			location: '/sign-in?redirect_to=%2Fchat%2F7&google_error=conflict',
		});
		expect(await thrown(back('?code=c&state=s', () => new Response('Your Google email isn’t verified yet.', { status: 400 })).run)).toMatchObject({
			location: '/sign-in?redirect_to=%2Fchat%2F7&google_error=unverified',
		});
		expect(await thrown(back('?code=c&state=s', () => new Response('This Google sign-in has expired. Try again.', { status: 400 })).run)).toMatchObject({
			location: '/sign-in?redirect_to=%2Fchat%2F7&google_error=expired',
		});
		expect(await thrown(back('?code=c&state=s', () => 'unreachable').run)).toMatchObject({ location: '/sign-in?redirect_to=%2Fchat%2F7&google_error=failed' });
	});

	it('goes back to Settings › General when Google was being added to a signed-in account', async () => {
		const linked = back('?code=c&state=s', () => json({ email: 'ana@gmail.com', is_new: false, linked: true, access_token: null, refresh_token: null }), { locals: { accessToken: 'a0' } });
		expect(await thrown(linked.run)).toMatchObject({ status: 303, location: '/settings/general?google=linked' });
		const refused = back('?code=c&state=s', () => new Response('That Google account already signs in to a different Nomi account.', { status: 409 }), { locals: { accessToken: 'a0' } });
		expect(await thrown(refused.run)).toMatchObject({ status: 303, location: '/settings/general?google_error=conflict' });
	});
});
