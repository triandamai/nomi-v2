import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json } from '../../../test-stubs/server';
import { actions } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const save = (form: Record<string, string>) => actions.password(event({ cookies: cookieJar({ access_token: 't' }), form }));

describe('account password', () => {
	it('sets a password, sending the current one only when given', async () => {
		const fetchFn = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchFn);
		expect(await save({ new_password: 'long-enough' })).toEqual({ passwordSaved: true });
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/auth\/password$/);
		expect(init.method).toBe('PUT');
		expect(JSON.parse(String(init.body))).toEqual({ current_password: null, new_password: 'long-enough' });
	});

	it('explains what the API refused', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ error: 'wrong_password' }, 403)));
		expect(await save({ current_password: 'x', new_password: 'long-enough' })).toMatchObject({ status: 403, data: { passwordError: expect.any(String) } });
		vi.stubGlobal('fetch', vi.fn(async () => json({ error: 'too_short' }, 400)));
		expect(await save({ new_password: 'short' })).toMatchObject({ status: 400 });
		expect(await save({ new_password: '' })).toMatchObject({ status: 400 });
	});
});

describe('account share links', () => {
	it('revokes a link', async () => {
		const fetchFn = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchFn);
		const result = await actions.revokeShare(event({ cookies: cookieJar({ access_token: 't' }), form: { id: 's', session_id: 'c' } }));
		expect(result).toEqual({ shareRevoked: true });
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/sessions\/c\/shares\/s$/);
		expect(init.method).toBe('DELETE');
	});

	it('says when it could not', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 404 })));
		expect(await actions.revokeShare(event({ cookies: cookieJar({ access_token: 't' }), form: { id: 's', session_id: 'c' } }))).toMatchObject({ status: 404 });
		expect(await actions.revokeShare(event({ cookies: cookieJar({ access_token: 't' }), form: {} }))).toMatchObject({ status: 400 });
	});
});

describe('account email change', () => {
	const cookies = () => cookieJar({ access_token: 't' });

	it('sends a code to the new address, then moves the account with it', async () => {
		const fetchFn = vi.fn(async () => json({ challenge_id: 'c1', email: 'n**@example.com' }));
		vi.stubGlobal('fetch', fetchFn);
		expect(await actions.emailStart(event({ cookies: cookies(), form: { email: ' new@example.com ' } }))).toEqual({ emailChallenge: 'c1', emailSentTo: 'n**@example.com' });
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/account\/email$/);
		expect(JSON.parse(String(init.body))).toEqual({ email: 'new@example.com' });

		const jar = cookies();
		vi.stubGlobal('fetch', vi.fn(async () => json({ email: 'new@example.com' })));
		expect(await actions.emailVerify(event({ cookies: jar, form: { challenge_id: 'c1', code: '123456' } }))).toEqual({ emailChanged: 'new@example.com' });
		expect(jar.get('user_email')).toBe('new@example.com');
	});

	it('keeps the code step open for a wrong code and explains a taken address', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ error: 'wrong_code', attempts_left: 4 }, 400)));
		expect(await actions.emailVerify(event({ cookies: cookies(), form: { challenge_id: 'c1', code: '000000' } }))).toMatchObject({
			status: 400,
			data: { emailChallenge: 'c1', emailError: expect.any(String) },
		});
		vi.stubGlobal('fetch', vi.fn(async () => json({ error: 'email_taken' }, 409)));
		expect(await actions.emailStart(event({ cookies: cookies(), form: { email: 'budi@example.com' } }))).toMatchObject({ status: 409 });
		expect(await actions.emailStart(event({ cookies: cookies(), form: { email: '' } }))).toMatchObject({ status: 400 });
	});
});

describe('account general preferences', () => {
	it('saves formats, who chats start with, Enter sends and reply tokens', async () => {
		const fetchFn = vi.fn(async () => json({}));
		vi.stubGlobal('fetch', fetchFn);
		const form = { formats: 'en-GB', default_agent: 'money', enter_sends: 'false', show_reply_tokens: 'true' };
		expect(await actions.general(event({ cookies: cookieJar({ access_token: 't' }), form }))).toEqual({ generalSaved: true });
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/preferences$/);
		expect(JSON.parse(String(init.body))).toEqual({ formats: 'en-GB', default_agent: 'money', enter_sends: false, show_reply_tokens: true });
		expect(await actions.general(event({ cookies: cookieJar({ access_token: 't' }), form: { ...form, formats: 'xx' } }))).toMatchObject({ status: 400 });
	});
});

describe('account deletion', () => {
	it('needs the box ticked, then deletes with the password and signs this browser out', async () => {
		const fetchFn = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchFn);
		expect(await actions.deleteAccount(event({ cookies: cookieJar({ access_token: 't' }), form: { password: 'pw' } }))).toMatchObject({ status: 400 });
		expect(fetchFn).not.toHaveBeenCalled();

		const jar = cookieJar({ access_token: 't', refresh_token: 'r', user_email: 'a@b.c' });
		await expect(actions.deleteAccount(event({ cookies: jar, form: { password: 'pw', confirm: 'on' } }))).rejects.toMatchObject({ status: 303, location: '/login' });
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/account$/);
		expect(init.method).toBe('DELETE');
		expect(JSON.parse(String(init.body))).toEqual({ password: 'pw' });
		expect(jar.get('access_token')).toBeUndefined();
	});

	it('emails a code, deletes with it, and names the workspaces to hand over', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ challenge_id: 'd1', email: 'a**@b.c' })));
		expect(await actions.deleteCode(event({ cookies: cookieJar({ access_token: 't' }) }))).toEqual({ deleteChallenge: 'd1', deleteSentTo: 'a**@b.c' });

		const fetchFn = vi.fn(async () => json({ error: 'sole_owner', orgs: [{ id: 'o', name: 'Acme' }] }, 409));
		vi.stubGlobal('fetch', fetchFn);
		const result = await actions.deleteAccount(event({ cookies: cookieJar({ access_token: 't' }), form: { challenge_id: 'd1', code: '123456', confirm: 'on' } }));
		expect(result).toMatchObject({ status: 409, data: { deleteError: expect.stringContaining('Acme') } });
		const [, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(JSON.parse(String(init.body))).toEqual({ challenge_id: 'd1', code: '123456' });
	});
});
