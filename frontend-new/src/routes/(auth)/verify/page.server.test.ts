import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

const challenge = { challenge_id: 'c1', purpose: 'login', email: 'k******@gmail.com', expires_in: 540, resend_in: 42, sends_left: 4 };
const pending = (overrides: Record<string, unknown> = {}) => JSON.stringify({ challengeId: 'c1', email: 'k••••••@gmail.com', isNew: false, redirectTo: '/chat/7', ...overrides });

function on(answer: Parameters<typeof fakeBackend>[0], options: { form?: Record<string, string>; pending?: string | null; locals?: Partial<App.Locals> } = {}) {
	const api = fakeBackend(answer);
	const cookies: Record<string, string> = options.pending === null ? {} : { pending_sign_in: options.pending ?? pending() };
	const event = fakeEvent({ url: 'http://localhost/verify', form: options.form, cookies, fetch: api.fetch, locals: options.locals });
	return { api, event };
}

describe('the code page', () => {
	it('shows where the code went, how long it works, and when another can be sent', async () => {
		const { api, event } = on(() => json(challenge));
		expect(await load(event as never)).toEqual({ expired: false, email: 'k••••••@gmail.com', minutes: 9, resendIn: 42, sendsLeft: 4, isNew: false, mailApp: 'gmail' });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/challenge/c1', method: 'GET' });
	});

	it('says the code has expired when the backend has let it go (after a reload too)', async () => {
		const { event } = on(() => json({ error: 'expired' }, 410), { pending: pending({ isNew: true }) });
		expect(await load(event as never)).toMatchObject({ expired: true, email: 'k••••••@gmail.com', isNew: true });
	});

	it('still takes the code when the backend can’t say how long it has', async () => {
		const { event } = on(() => 'unreachable');
		expect(await load(event as never)).toMatchObject({ expired: false, minutes: null, resendIn: 0, sendsLeft: 1 });
	});

	it('sends anyone without a sign-in waiting back to sign in, or on into the app', async () => {
		expect(await thrown(() => load(on(() => json({}), { pending: null }).event as never))).toMatchObject({ status: 303, location: '/sign-in' });
		expect(await thrown(() => load(on(() => json({}), { pending: null, locals: { accessToken: 'a1' } }).event as never))).toMatchObject({ status: 303, location: '/' });
	});
});

describe('entering the code', () => {
	const tokens = (call: { url: string }) => (call.url.endsWith('/verify') ? json({ access_token: 'a1', refresh_token: 'r1' }) : json({ language: 'id' }));

	it('signs in and goes where the person was going', async () => {
		const { api, event } = on(tokens, { form: { code: '048 917' } });
		expect(await thrown(() => actions.verify(event as never))).toMatchObject({ status: 303, location: '/chat/7' });
		expect(api.calls[0]).toEqual({ url: 'http://api.test/api/auth/verify', method: 'POST', body: { challenge_id: 'c1', code: '048917' }, headers: { 'content-type': 'application/json' } });
		expect(event.jar.get('access_token')).toBe('a1');
		expect(event.jar.has('pending_sign_in')).toBe(false);
		expect(event.jar.get('nm_last_sign_in')).toBe('email');
		expect(event.jar.get('PARAGLIDE_LOCALE')).toBe('id');
	});

	it('takes a new account to its first run, then on to where it was going', async () => {
		const { event } = on(tokens, { form: { code: '048917' }, pending: pending({ isNew: true, redirectTo: '/invite/Xk29' }) });
		expect(await thrown(() => actions.verify(event as never))).toMatchObject({ status: 303, location: '/first-run?redirect_to=%2Finvite%2FXk29' });
	});

	it('wants all six digits before asking the backend', async () => {
		const { api, event } = on(tokens, { form: { code: '0489' } });
		expect(await actions.verify(event as never)).toMatchObject({ status: 400, data: { problem: 'incomplete' } });
		expect(api.calls).toEqual([]);
	});

	it('says how many tries are left after a wrong code', async () => {
		const { event } = on(() => json({ error: 'wrong_code', attempts_left: 3 }, 400), { form: { code: '111111' } });
		expect(await actions.verify(event as never)).toMatchObject({ status: 400, data: { problem: 'wrong_code', attemptsLeft: 3 } });
	});

	it('locks after too many wrong codes, and knows an expired one', async () => {
		expect(await actions.verify(on(() => json({ error: 'too_many_attempts' }, 410), { form: { code: '111111' } }).event as never)).toMatchObject({ status: 410, data: { problem: 'locked' } });
		expect(await actions.verify(on(() => json({ error: 'expired' }, 410), { form: { code: '111111' } }).event as never)).toMatchObject({ status: 410, data: { problem: 'expired' } });
		expect(await actions.verify(on(() => json({ error: 'failed' }, 500), { form: { code: '111111' } }).event as never)).toMatchObject({ status: 500, data: { problem: 'failed' } });
		expect(await actions.verify(on(() => 'unreachable', { form: { code: '111111' } }).event as never)).toMatchObject({ status: 503, data: { problem: 'unreachable' } });
	});
});

describe('asking for a new code', () => {
	it('restarts the countdown with the backend’s numbers', async () => {
		const { api, event } = on(() => json({ ...challenge, resend_in: 60, sends_left: 3 }));
		expect(await actions.resend(event as never)).toEqual({ resent: true, resendIn: 60, sendsLeft: 3 });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/resend', body: { challenge_id: 'c1' } });
	});

	it('says to wait when it is too soon, for as long as the backend says', async () => {
		expect(await actions.resend(on(() => json({ error: 'wait', seconds: 37 }, 429)).event as never)).toMatchObject({ status: 429, data: { problem: 'wait', seconds: 37 } });
	});

	it('knows when no more codes can go out, the sign-in expired, or the email failed', async () => {
		expect(await actions.resend(on(() => json({ error: 'too_many_codes' }, 429)).event as never)).toMatchObject({ status: 429, data: { problem: 'too_many_codes', sendsLeft: 0 } });
		expect(await actions.resend(on(() => json({ error: 'expired' }, 410)).event as never)).toMatchObject({ status: 410, data: { problem: 'expired' } });
		expect(await actions.resend(on(() => json({ error: 'email_failed' }, 502)).event as never)).toMatchObject({ status: 502, data: { problem: 'email_failed' } });
	});
});

describe('changing the email', () => {
	it('goes back to the form it came from, keeping where the person was going', async () => {
		const signIn = on(() => json({}));
		expect(await thrown(() => actions.cancel(signIn.event as never))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Fchat%2F7' });
		expect(signIn.event.jar.has('pending_sign_in')).toBe(false);
		const register = on(() => json({}), { pending: pending({ isNew: true, redirectTo: '/' }) });
		expect(await thrown(() => actions.cancel(register.event as never))).toMatchObject({ status: 303, location: '/register' });
	});
});
