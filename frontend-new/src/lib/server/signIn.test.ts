import { describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { clearPendingSignIn, completeSignIn, getPendingSignIn, safeRedirect, setPendingSignIn } = await import('./signIn');

function cookieJar(initial: Record<string, string>) {
	const jar = new Map(Object.entries(initial));
	const options = new Map<string, Record<string, unknown>>();
	const cookies = {
		get: (name: string) => jar.get(name),
		set: (name: string, value: string, opts: Record<string, unknown>) => {
			jar.set(name, value);
			options.set(name, opts);
		},
		delete: (name: string) => void jar.delete(name),
	} as unknown as Cookies;
	return { jar, options, cookies };
}

describe('the sign-in waiting for its emailed code', () => {
	it('is kept on the server for 15 minutes, then read back', () => {
		const { options, cookies } = cookieJar({});
		const pending = { challengeId: 'c1', email: 'ko••••@gmail.com', isNew: true, redirectTo: '/projects' };
		setPendingSignIn(cookies, pending);
		expect(getPendingSignIn(cookies)).toEqual(pending);
		expect(options.get('pending_sign_in')).toMatchObject({ httpOnly: true, maxAge: 15 * 60, path: '/' });
		clearPendingSignIn(cookies);
		expect(getPendingSignIn(cookies)).toBeNull();
	});

	it('ignores a cookie it did not write', () => {
		for (const raw of ['not json', '{"email":"x"}']) expect(getPendingSignIn(cookieJar({ pending_sign_in: raw }).cookies)).toBeNull();
	});
});

describe('safeRedirect', () => {
	it('only ever sends someone to a page on this site', () => {
		expect(safeRedirect('/chat/abc?x=1')).toBe('/chat/abc?x=1');
		for (const target of ['https://evil.test', '//evil.test', 'javascript:alert(1)', '', null, undefined]) expect(safeRedirect(target)).toBe('/');
	});
});

describe('completeSignIn', () => {
	function backend(preferences: Record<string, unknown>) {
		const calls: { url: string; method: string; body?: unknown }[] = [];
		const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
			calls.push({ url: String(input), method: init?.method ?? 'GET', body: init?.body ? JSON.parse(String(init.body)) : undefined });
			return init?.method === 'PUT' ? new Response(null, { status: 204 }) : Response.json(preferences);
		});
		return { calls, fetch: fetchFn as unknown as typeof fetch };
	}

	it('signs an existing account in and keeps the language it already chose', async () => {
		const api = backend({ language: 'id' });
		const { jar, cookies } = cookieJar({});
		await completeSignIn({ fetch: api.fetch, cookies, locals: {} }, { access_token: 'a1', refresh_token: 'r1' }, false);
		expect(jar.get('access_token')).toBe('a1');
		expect(jar.get('refresh_token')).toBe('r1');
		expect(jar.get('PARAGLIDE_LOCALE')).toBe('id');
		expect(api.calls.map((call) => [call.method, call.url])).toEqual([['GET', 'http://api.test/api/preferences']]);
	});

	it('gives a brand-new account the language its sign-up pages were in', async () => {
		const api = backend({});
		const { cookies } = cookieJar({});
		await completeSignIn({ fetch: api.fetch, cookies, locals: {} }, { access_token: 'a1', refresh_token: 'r1' }, true);
		expect(api.calls).toEqual([{ url: 'http://api.test/api/preferences', method: 'PUT', body: { language: 'en' } }]);
	});
});
