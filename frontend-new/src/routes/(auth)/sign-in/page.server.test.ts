import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

const verification = { challenge_id: 'c1', purpose: 'login', email: 'a**@gmail.com', expires_in: 600, resend_in: 60, sends_left: 5 };
const signIn = (form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0], url = 'http://localhost/sign-in') => {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url, form, fetch: api.fetch });
	return { api, event, run: () => actions.default(event as never) };
};

describe('sign in: the form', () => {
	it('asks for an email that looks like one, and a password, before calling the backend', async () => {
		for (const [form, fields] of [
			[{ email: '', password: 'x' }, { email: 'email_required' }],
			[{ email: 'ana', password: 'x' }, { email: 'email_invalid' }],
			[{ email: 'ana@example.com', password: '' }, { password: 'password_required' }],
			[{ email: '', password: '' }, { email: 'email_required', password: 'password_required' }],
		] as const) {
			const { api, run } = signIn(form, () => json({}));
			expect(await run()).toMatchObject({ status: 400, data: { email: form.email, fields, alert: null } });
			expect(api.calls).toEqual([]);
		}
	});

	it('points a wrong email or password at the password, keeping the email', async () => {
		const { api, run } = signIn({ email: ' ana@example.com ', password: 'nope' }, () => json({ error: 'invalid credentials' }, 401));
		expect(await run()).toMatchObject({ status: 401, data: { email: 'ana@example.com', fields: { password: 'wrong_credentials' }, alert: null } });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/login', method: 'POST', body: { email: 'ana@example.com', password: 'nope' } });
	});

	it('says when too many codes went out, or the code email failed, or anything else', async () => {
		expect(await signIn({ email: 'ana@example.com', password: 'pw' }, () => json({ error: 'too_many_codes' }, 429)).run()).toMatchObject({
			status: 429,
			data: { fields: {}, alert: { kind: 'too_many_codes', status: 429, code: 'too_many_codes' } },
		});
		expect(await signIn({ email: 'ana@example.com', password: 'pw' }, () => json({ error: 'email_failed' }, 502)).run()).toMatchObject({
			status: 502,
			data: { alert: { kind: 'email_failed', status: 502, code: 'email_failed' } },
		});
		expect(await signIn({ email: 'ana@example.com', password: 'pw' }, () => json({ error: 'login failed' }, 500)).run()).toMatchObject({
			status: 500,
			data: { alert: { kind: 'sign_in_failed', status: 500 } },
		});
		expect(await signIn({ email: 'ana@example.com', password: 'pw' }, () => 'unreachable').run()).toMatchObject({ status: 503, data: { alert: { kind: 'unreachable' } } });
	});

	it('goes on to the code with the right password, remembering where the person was going', async () => {
		const { event, run } = signIn({ email: 'ana@gmail.com', password: 'pw' }, () => json({ verification }), 'http://localhost/sign-in?redirect_to=%2Fchat%2F7');
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/verify' });
		expect(JSON.parse(event.jar.get('pending_sign_in')!)).toEqual({ challengeId: 'c1', email: 'a••@gmail.com', isNew: false, redirectTo: '/chat/7' });
		expect(event.jar.get('access_token')).toBeUndefined();
	});

	it('signs straight in when the server has emailed codes off', async () => {
		const { event, run } = signIn({ email: 'ana@example.com', password: 'pw' }, (call) =>
			call.url.endsWith('/login') ? json({ access_token: 'a1', refresh_token: 'r1' }) : json({ language: 'en' }),
		);
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/' });
		expect(event.jar.get('access_token')).toBe('a1');
		expect(event.jar.get('nm_last_sign_in')).toBe('email');
	});

	it('never sends someone to another site after signing in', async () => {
		const { event, run } = signIn({ email: 'ana@gmail.com', password: 'pw' }, () => json({ verification }), 'http://localhost/sign-in?redirect_to=https%3A%2F%2Fevil.test');
		await thrown(run);
		expect(JSON.parse(event.jar.get('pending_sign_in')!).redirectTo).toBe('/');
	});
});

describe('sign in: the page', () => {
	const page = (url: string, answer: Parameters<typeof fakeBackend>[0] = () => json({ available: false }), extra: Parameters<typeof fakeEvent>[0] = {}) =>
		load(fakeEvent({ url, fetch: fakeBackend(answer).fetch, ...extra }) as never) as Promise<Record<string, unknown>>;

	it('offers Google only when the server has it, marked when it was the last way in', async () => {
		expect(await page('http://localhost/sign-in')).toMatchObject({ google: false, lastUsed: null });
		expect(await page('http://localhost/sign-in', () => json({ available: true }), { cookies: { nm_last_sign_in: 'google' } })).toMatchObject({ google: true, lastUsed: 'google' });
	});

	it('says why Google came back, and that a new password is ready', async () => {
		expect(await page('http://localhost/sign-in?google_error=conflict')).toMatchObject({ googleError: 'conflict', passwordChanged: false });
		expect(await page('http://localhost/sign-in?google_error=whatever')).toMatchObject({ googleError: 'failed' });
		expect(await page('http://localhost/sign-in?reset=1')).toMatchObject({ googleError: null, passwordChanged: true });
	});

	it('keeps where the person is going and the email they came with', async () => {
		expect(await page('http://localhost/sign-in?redirect_to=%2Finvite%2Fabc&email=ana%40example.com')).toMatchObject({ redirectTo: '/invite/abc', email: 'ana@example.com' });
		expect(await page('http://localhost/sign-in?redirect_to=%2F%2Fevil.test')).toMatchObject({ redirectTo: null, email: '' });
	});

	it('sends someone already signed in on to where they were going', async () => {
		const event = fakeEvent({ url: 'http://localhost/sign-in?redirect_to=%2Fchats', locals: { accessToken: 'a1' } });
		expect(await thrown(() => load(event as never))).toMatchObject({ status: 303, location: '/chats' });
	});
});
