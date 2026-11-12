import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { alertFor, callAuth, emailProblem, googleAvailable, googleErrorFor, lastMethod, newPasswordProblem, rememberMethod } = await import('./authFlow');

describe('callAuth', () => {
	it('posts JSON to the backend, without asking for problem details (they drop tries and waits)', async () => {
		const api = fakeBackend(() => json({ ok: true }));
		await callAuth(api.fetch, '/api/auth/verify', { challenge_id: 'c1', code: '123456' });
		expect(api.calls).toEqual([
			{ url: 'http://api.test/api/auth/verify', method: 'POST', body: { challenge_id: 'c1', code: '123456' }, headers: { 'content-type': 'application/json' } },
		]);
	});

	it('gets when there is no body, and says null when the backend cannot be reached', async () => {
		const api = fakeBackend(() => 'unreachable');
		expect(await callAuth(api.fetch, '/api/auth/challenge/c1')).toBeNull();
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/challenge/c1', method: 'GET' });
	});
});

describe('googleAvailable', () => {
	it('offers Google only when the server says it is set up', async () => {
		expect(await googleAvailable(fakeBackend(() => json({ available: true })).fetch)).toBe(true);
		expect(await googleAvailable(fakeBackend(() => json({ available: false })).fetch)).toBe(false);
		expect(await googleAvailable(fakeBackend(() => 'unreachable').fetch)).toBe(false);
		expect(await googleAvailable(fakeBackend(() => json({}, 500)).fetch)).toBe(false);
	});
});

describe('the way this browser signed in last', () => {
	it('is remembered for a year, and only Google or email count', () => {
		const event = fakeEvent({});
		expect(lastMethod(event.cookies)).toBeNull();
		rememberMethod(event.cookies, 'google');
		expect(lastMethod(event.cookies)).toBe('google');
		expect(event.cookieOptions.get('nm_last_sign_in')).toMatchObject({ httpOnly: true, path: '/', maxAge: 365 * 24 * 60 * 60 });
		expect(lastMethod(fakeEvent({ cookies: { nm_last_sign_in: 'facebook' } }).cookies)).toBeNull();
	});
});

describe('googleErrorFor', () => {
	it('names why Google sign-in came back without signing in', async () => {
		expect(await googleErrorFor(new Response('That Google account already signs in to a different Nomi account.', { status: 409 }))).toBe('conflict');
		expect(await googleErrorFor(new Response('Google sign-in isn’t set up on this server yet.', { status: 503 }))).toBe('unavailable');
		expect(await googleErrorFor(new Response('Your Google email isn’t verified yet. Verify it with Google, then try again.', { status: 400 }))).toBe('unverified');
		expect(await googleErrorFor(new Response('That invite code isn’t valid any more.', { status: 400 }))).toBe('invite');
		expect(await googleErrorFor(new Response('This Google sign-in has expired. Try again.', { status: 400 }))).toBe('expired');
		expect(await googleErrorFor(new Response('Google didn’t accept the sign-in. Try again.', { status: 400 }))).toBe('expired');
		expect(await googleErrorFor(new Response('Google sign-in failed. Try again.', { status: 500 }))).toBe('failed');
	});
});

describe('alertFor', () => {
	it('tells too many codes and an email that did not go out from any other failure', async () => {
		expect(await alertFor(json({ error: 'too_many_codes' }, 429), 'sign_in_failed')).toEqual({ kind: 'too_many_codes', status: 429, code: 'too_many_codes' });
		expect(await alertFor(json({ error: 'email_failed' }, 502), 'sign_in_failed')).toEqual({ kind: 'email_failed', status: 502, code: 'email_failed' });
		expect(await alertFor(json({ error: 'login failed' }, 500), 'sign_in_failed')).toEqual({ kind: 'sign_in_failed', status: 500 });
	});
});

describe('the fields every sign-in screen checks first', () => {
	it('wants an email that looks like one', () => {
		expect(emailProblem('')).toBe('email_required');
		expect(emailProblem('ana')).toBe('email_invalid');
		expect(emailProblem('ana@example.com')).toBeNull();
	});

	it('wants a new password of at least 8 characters', () => {
		expect(newPasswordProblem('')).toBe('password_required');
		expect(newPasswordProblem('1234567')).toBe('password_short');
		expect(newPasswordProblem('12345678')).toBeNull();
	});
});
