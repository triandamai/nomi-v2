import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

const verification = { challenge_id: 'c9', purpose: 'register', email: 'n******@nomi.test', expires_in: 600, resend_in: 60, sends_left: 5 };
const register = (form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0], url = 'http://localhost/register') => {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url, form, fetch: api.fetch });
	return { api, event, run: () => actions.default(event as never) };
};
const filled = { email: 'new@nomi.test', password: 'correct-horse', workspace: 'Family budget' };

describe('create an account: the form', () => {
	it('asks for an email, a password of 8 or more and a workspace name first', async () => {
		for (const [form, fields] of [
			[{ ...filled, email: 'nope' }, { email: 'email_invalid' }],
			[{ ...filled, password: '' }, { password: 'password_required' }],
			[{ ...filled, password: '1234567' }, { password: 'password_short' }],
			[{ ...filled, workspace: '  ' }, { workspace: 'workspace_required' }],
		] as const) {
			const { api, run } = register(form, () => json({}));
			expect(await run()).toMatchObject({ status: 400, data: { fields, alert: null } });
			expect(api.calls).toEqual([]);
		}
	});

	it('makes the account with its own workspace, in the language the page was in, and goes on to the code', async () => {
		const { api, event, run } = register(filled, () => json({ verification }), 'http://localhost/register?redirect_to=%2Fprojects');
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/verify' });
		expect(api.calls[0]).toMatchObject({
			url: 'http://api.test/api/auth/register',
			body: { email: 'new@nomi.test', password: 'correct-horse', org: { mode: 'create', name: 'Family budget' }, language: 'en' },
		});
		expect(JSON.parse(event.jar.get('pending_sign_in')!)).toEqual({ challengeId: 'c9', email: 'n••••••@nomi.test', isNew: true, redirectTo: '/projects' });
	});

	it('joins the workspace an invite is for instead, without asking for a name', async () => {
		const { api, run } = register({ email: 'new@nomi.test', password: 'correct-horse' }, () => json({ verification }), 'http://localhost/register?redirect_to=%2Finvite%2FXk29&email=new%40nomi.test');
		await thrown(run);
		expect(api.calls[0].body).toMatchObject({ org: { mode: 'join', invite_code: 'Xk29' } });
	});

	it('says the email is taken, or the invite is spent, on their fields', async () => {
		expect(await register(filled, () => new Response('email already registered', { status: 409 })).run()).toMatchObject({
			status: 409,
			data: { values: { email: 'new@nomi.test', workspace: 'Family budget' }, fields: { email: 'email_taken' } },
		});
		expect(
			await register({ email: 'new@nomi.test', password: 'correct-horse' }, () => new Response('invite code not found, expired, or already used', { status: 400 }), 'http://localhost/register?redirect_to=%2Finvite%2FXk29').run(),
		).toMatchObject({ status: 400, data: { fields: { invite: 'invite_invalid' } } });
	});

	it('says the account exists when only the code email failed, and anything else plainly', async () => {
		expect(await register(filled, () => new Response('couldn’t send the confirmation code', { status: 502 })).run()).toMatchObject({
			status: 502,
			data: { alert: { kind: 'register_email_failed', status: 502 } },
		});
		expect(await register(filled, () => new Response('registration failed', { status: 500 })).run()).toMatchObject({ status: 500, data: { alert: { kind: 'register_failed', status: 500 } } });
		expect(await register(filled, () => 'unreachable').run()).toMatchObject({ status: 503, data: { alert: { kind: 'unreachable' } } });
	});

	it('signs a new account in straight away when the server has codes off, then asks its name', async () => {
		const { event, run } = register(filled, (call) => (call.url.endsWith('/register') ? json({ access_token: 'a1', refresh_token: 'r1' }) : new Response(null, { status: 204 })));
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/first-run' });
		expect(event.jar.get('access_token')).toBe('a1');
	});
});

describe('create an account: the page', () => {
	it('knows the invite it came from and the address it was sent to', async () => {
		const event = fakeEvent({ url: 'http://localhost/register?redirect_to=%2Finvite%2FXk29&email=new%40nomi.test', fetch: fakeBackend(() => json({ available: true })).fetch });
		expect(await load(event as never)).toMatchObject({ google: true, invite: 'Xk29', email: 'new@nomi.test', redirectTo: '/invite/Xk29' });
	});

	it('has no invite otherwise, and sends someone signed in on', async () => {
		expect(await load(fakeEvent({ url: 'http://localhost/register', fetch: fakeBackend(() => json({ available: false })).fetch }) as never)).toMatchObject({ invite: null, email: '' });
		expect(await thrown(() => load(fakeEvent({ url: 'http://localhost/register', locals: { accessToken: 'a1' } }) as never))).toMatchObject({ status: 303, location: '/' });
	});
});
