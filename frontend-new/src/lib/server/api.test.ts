import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { apiFetch, authenticate, SIGN_IN_PATH } = await import('./api');

function jwt(exp: number): string {
	const part = (value: object) => Buffer.from(JSON.stringify(value)).toString('base64url');
	return `${part({ alg: 'HS256' })}.${part({ exp })}.sig`;
}

/** Cookies as SvelteKit keeps them for one request. */
function cookieJar(initial: Record<string, string>) {
	const jar = new Map(Object.entries(initial));
	const cookies = {
		get: (name: string) => jar.get(name),
		set: (name: string, value: string) => void jar.set(name, value),
		delete: (name: string) => void jar.delete(name),
	} as unknown as Cookies;
	return { jar, cookies };
}

type Call = { url: string; authorization: string | null; body?: string };

/** A fake backend: accepts `validToken`, refreshes to `refreshTo` (or 401s when null). */
function backend(options: { validToken: string; refreshTo: string | null }) {
	const calls: Call[] = [];
	const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
		const url = String(input);
		const headers = new Headers(init?.headers);
		calls.push({ url, authorization: headers.get('authorization'), body: init?.body as string | undefined });
		if (url.endsWith('/api/auth/refresh')) {
			return options.refreshTo
				? Response.json({ access_token: options.refreshTo })
				: new Response(null, { status: 401 });
		}
		await new Promise((resolve) => setTimeout(resolve, 5));
		return headers.get('authorization') === `Bearer ${options.validToken}` ? Response.json({ ok: true }) : new Response(null, { status: 401 });
	});
	return { calls, fetch: fetchFn as unknown as typeof fetch };
}

let soon: number;
beforeEach(() => {
	soon = Math.floor(Date.now() / 1000) + 3600;
});

describe('apiFetch', () => {
	it('calls the backend as the person, with JSON', async () => {
		const api = backend({ validToken: 'a1', refreshTo: null });
		const { cookies } = cookieJar({ access_token: 'a1' });
		const response = await apiFetch({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, '/api/profile');
		expect(response.status).toBe(200);
		expect(api.calls).toEqual([{ url: 'http://api.test/api/profile', authorization: 'Bearer a1', body: undefined }]);
	});

	it('refreshes an expired token once and retries', async () => {
		const api = backend({ validToken: 'a2', refreshTo: 'a2' });
		const { jar, cookies } = cookieJar({ access_token: 'a1', refresh_token: 'r1' });
		const response = await apiFetch({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, '/api/profile');
		expect(response.status).toBe(200);
		expect(api.calls.map((call) => [call.url, call.authorization])).toEqual([
			['http://api.test/api/profile', 'Bearer a1'],
			['http://api.test/api/auth/refresh', null],
			['http://api.test/api/profile', 'Bearer a2'],
		]);
		expect(jar.get('access_token')).toBe('a2');
		expect(jar.get('refresh_token')).toBe('r1');
	});

	it('refreshes only once when a page load makes several calls at the same time', async () => {
		const api = backend({ validToken: 'a2', refreshTo: 'a2' });
		const { cookies } = cookieJar({ access_token: 'a1', refresh_token: 'r1' });
		const event = { fetch: api.fetch, cookies, locals: { accessToken: 'a1' } };
		const responses = await Promise.all(['/api/profile', '/api/preferences', '/api/agents', '/api/usage/brief', '/api/notifications/unread'].map((path) => apiFetch(event, path)));
		expect(responses.map((response) => response.status)).toEqual([200, 200, 200, 200, 200]);
		expect(api.calls.filter((call) => call.url.endsWith('/api/auth/refresh'))).toHaveLength(1);
	});

	it('signs the person out when the refresh token is refused too', async () => {
		const api = backend({ validToken: 'never', refreshTo: null });
		const { jar, cookies } = cookieJar({ access_token: 'a1', refresh_token: 'r1' });
		await expect(apiFetch({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, '/api/profile')).rejects.toMatchObject({
			status: 303,
			location: SIGN_IN_PATH,
		});
		expect(jar.has('access_token')).toBe(false);
		expect(jar.has('refresh_token')).toBe(false);
	});
});

describe('authenticate', () => {
	it('uses a token that is still good, without asking the backend', async () => {
		const api = backend({ validToken: 'x', refreshTo: 'unused' });
		const token = jwt(soon);
		const { cookies } = cookieJar({ access_token: token, refresh_token: 'r1' });
		const locals: App.Locals = {};
		await authenticate({ fetch: api.fetch, cookies, locals });
		expect(locals.accessToken).toBe(token);
		expect(api.calls).toEqual([]);
	});

	it('refreshes a token about to expire (or gone) before the page loads', async () => {
		const api = backend({ validToken: 'x', refreshTo: 'fresh' });
		const sessions: Record<string, string>[] = [{ access_token: jwt(Math.floor(Date.now() / 1000) + 10), refresh_token: 'r1' }, { refresh_token: 'r1' }];
		for (const initial of sessions) {
			const { cookies } = cookieJar(initial);
			const locals: App.Locals = {};
			await authenticate({ fetch: api.fetch, cookies, locals });
			expect(locals.accessToken).toBe('fresh');
		}
	});

	it('leaves a signed-out visitor signed out', async () => {
		const api = backend({ validToken: 'x', refreshTo: 'unused' });
		const { cookies } = cookieJar({});
		const locals: App.Locals = {};
		await authenticate({ fetch: api.fetch, cookies, locals });
		expect(locals.accessToken).toBeUndefined();
		expect(api.calls).toEqual([]);
	});
});
