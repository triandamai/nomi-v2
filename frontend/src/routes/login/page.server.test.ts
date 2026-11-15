import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json, settle } from '../../test-stubs/server';
import { getPendingSignIn } from '$lib/server/signIn';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const reply = (status: number) => vi.stubGlobal('fetch', vi.fn(async () => new Response('{}', { status })));

describe('login', () => {
	it('points a failed sign-in at the password field', async () => {
		reply(401);
		expect(await actions.default(event({ form: { email: 'ana@example.com', password: 'nope' } }))).toMatchObject({ status: 401, data: { field: 'password' } });
		expect(await actions.default(event({ form: { email: '', password: 'x' } }))).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(await actions.default(event({ form: { email: 'ana@example.com', password: '' } }))).toMatchObject({ status: 400, data: { field: 'password' } });
	});

	it('keeps errors that are not about a field out of the fields', async () => {
		for (const status of [429, 502, 500]) {
			reply(status);
			expect(await actions.default(event({ form: { email: 'ana@example.com', password: 'x' } }))).toMatchObject({ status, data: { field: null } });
		}
	});

	it('knows when the person is back from choosing a new password', async () => {
		reply(404);
		expect(await load(event({ url: 'http://localhost/login?reset=1' }))).toMatchObject({ passwordChanged: true });
		expect(await load(event({ url: 'http://localhost/login' }))).toMatchObject({ passwordChanged: false });
	});

	it('emails a code with the address alone and continues on /verify', async () => {
		const fetchFn = vi.fn(async () =>
			json({ verification: { challenge_id: 'c1', purpose: 'register', email: 'n**@example.com', expires_in: 600, resend_in: 60, sends_left: 4 } }),
		);
		vi.stubGlobal('fetch', fetchFn);
		const cookies = cookieJar();
		const res = await settle(() =>
			actions.default(event({ cookies, url: 'http://localhost/login?redirect_to=/chat', form: { email: ' nia@example.com ', password: '', mode: 'code' } })),
		);
		expect(res).toMatchObject({ thrown: { status: 303, location: '/verify' } });
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/auth\/start$/);
		expect(JSON.parse(String(init.body))).toMatchObject({ email: 'nia@example.com' });
		expect(getPendingSignIn(cookies)).toEqual({ challengeId: 'c1', email: 'nia@example.com', isNew: false, redirectTo: '/chat' });
	});

	it('asks for the address before emailing a code', async () => {
		reply(400);
		expect(await actions.default(event({ form: { email: '', mode: 'code' } }))).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(await actions.default(event({ form: { email: 'nope', mode: 'code' } }))).toMatchObject({ status: 400, data: { field: 'email' } });
	});
});
