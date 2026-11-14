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
