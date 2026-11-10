import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { actions, load } = await import('./+page.server');

describe('the reset link', () => {
	const open = (url: string, answer: Parameters<typeof fakeBackend>[0]) => {
		const api = fakeBackend(answer);
		return { api, run: () => load(fakeEvent({ url, fetch: api.fetch }) as never) };
	};

	it('asks for a new password while the link still works', async () => {
		const { api, run } = open('http://localhost/reset-password?token=t0k3n', () => new Response(null, { status: 204 }));
		expect(await run()).toEqual({ token: 't0k3n', expired: false });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/reset/t0k3n', method: 'GET' });
	});

	it('says it has expired when the backend says so, or when there is no token', async () => {
		expect(await open('http://localhost/reset-password?token=old', () => json({ error: 'expired' }, 410)).run()).toEqual({ token: 'old', expired: true });
		expect(await open('http://localhost/reset-password', () => json({})).run()).toEqual({ token: '', expired: true });
	});

	it('lets them try when the backend only hiccuped', async () => {
		expect(await open('http://localhost/reset-password?token=t', () => 'unreachable').run()).toEqual({ token: 't', expired: false });
	});
});

describe('choosing the new password', () => {
	const choose = (form: Record<string, string>, answer: Parameters<typeof fakeBackend>[0]) => {
		const api = fakeBackend(answer);
		const event = fakeEvent({ url: 'http://localhost/reset-password?token=t0k3n', form, fetch: api.fetch, cookies: { access_token: 'old', refresh_token: 'old' } });
		return { api, event, run: () => actions.default(event as never) };
	};

	it('wants 8 characters or more, typed the same twice', async () => {
		for (const [form, fields] of [
			[{ token: 't', password: 'short', confirm: 'short' }, { password: 'password_short' }],
			[{ token: 't', password: '', confirm: '' }, { password: 'password_required' }],
			[{ token: 't', password: 'long enough', confirm: 'long enougH' }, { confirm: 'password_mismatch' }],
		] as const) {
			const { api, run } = choose(form, () => new Response(null, { status: 204 }));
			expect(await run()).toMatchObject({ status: 400, data: { fields, expired: false } });
			expect(api.calls).toEqual([]);
		}
	});

	it('saves it, signs this browser out as every other, and sends them to sign in with a note', async () => {
		const { api, event, run } = choose({ token: 't0k3n', password: 'long enough', confirm: 'long enough' }, () => new Response(null, { status: 204 }));
		expect(await thrown(run)).toMatchObject({ status: 303, location: '/sign-in?reset=1' });
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/auth/reset', body: { token: 't0k3n', password: 'long enough' } });
		expect(event.jar.has('access_token')).toBe(false);
		expect(event.jar.has('refresh_token')).toBe(false);
	});

	it('knows a link that ran out meanwhile, a password the backend finds short, and any other failure', async () => {
		const good = { token: 't', password: 'long enough', confirm: 'long enough' };
		expect(await choose(good, () => json({ error: 'expired' }, 410)).run()).toMatchObject({ status: 410, data: { expired: true } });
		expect(await choose({ ...good, token: '' }, () => json({})).run()).toMatchObject({ status: 410, data: { expired: true } });
		expect(await choose(good, () => json({ error: 'too_short' }, 400)).run()).toMatchObject({ status: 400, data: { fields: { password: 'password_short' } } });
		expect(await choose(good, () => json({ error: 'failed' }, 500)).run()).toMatchObject({ status: 500, data: { alert: { kind: 'reset_failed', status: 500 } } });
		expect(await choose(good, () => 'unreachable').run()).toMatchObject({ status: 503, data: { alert: { kind: 'unreachable' } } });
	});
});
