import { describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { createWorkspace, savePreferences, signOut, signOutTarget, chooseProjectsWorkspace } = await import('./account');

function cookieJar(initial: Record<string, string> = {}) {
	const jar = new Map(Object.entries(initial));
	const cookies = {
		get: (name: string) => jar.get(name),
		set: (name: string, value: string) => void jar.set(name, value),
		delete: (name: string) => void jar.delete(name),
	} as unknown as Cookies;
	return { jar, cookies };
}

function backend(answer: (method: string, path: string, body: unknown) => Response | Promise<Response>) {
	const calls: { method: string; path: string; body?: unknown; authorization?: string | null }[] = [];
	const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
		const path = new URL(String(input)).pathname;
		const method = init?.method ?? 'GET';
		const body = init?.body ? JSON.parse(String(init.body)) : undefined;
		calls.push({ method, path, ...(body !== undefined && { body }), authorization: new Headers(init?.headers).get('authorization') });
		return answer(method, path, body);
	});
	return { calls, fetch: fetchFn as unknown as typeof fetch };
}

describe('signOut', () => {
	it('ends the sign-in on the backend and forgets it on this device', async () => {
		const api = backend(() => new Response(null, { status: 204 }));
		const { jar, cookies } = cookieJar({ access_token: 'a1', refresh_token: 'r1', device_id: 'd1' });
		await signOut({ fetch: api.fetch, cookies, locals: {} });
		expect(api.calls).toEqual([{ method: 'POST', path: '/api/auth/logout', body: { refresh_token: 'r1' }, authorization: null }]);
		expect([...jar.keys()]).toEqual(['device_id']);
	});

	it('forgets it here even when the backend can’t be reached', async () => {
		const api = backend(() => Promise.reject(new Error('down')));
		const { jar, cookies } = cookieJar({ access_token: 'a1', refresh_token: 'r1' });
		await signOut({ fetch: api.fetch, cookies, locals: {} });
		expect(jar.size).toBe(0);
	});

	it('has nothing to revoke without a refresh token', async () => {
		const api = backend(() => new Response(null, { status: 204 }));
		await signOut({ fetch: api.fetch, cookies: cookieJar({ access_token: 'a1' }).cookies, locals: {} });
		expect(api.calls).toEqual([]);
	});
});

describe('signOutTarget', () => {
	it('goes to sign in, or back to a page on this site', () => {
		expect(signOutTarget(null)).toBe('/sign-in');
		expect(signOutTarget('/sign-in?redirect_to=%2Fchats')).toBe('/sign-in?redirect_to=%2Fchats');
		expect(signOutTarget('https://evil.test')).toBe('/sign-in');
		expect(signOutTarget('//evil.test')).toBe('/sign-in');
	});
});

describe('savePreferences', () => {
	it('saves the theme and keeps it on this device', async () => {
		const api = backend(() => Response.json({ theme: 'light' }));
		const { jar, cookies } = cookieJar();
		expect(await savePreferences({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, { theme: 'light' })).toEqual({ ok: true });
		expect(api.calls).toEqual([{ method: 'PUT', path: '/api/preferences', body: { theme: 'light' }, authorization: 'Bearer a1' }]);
		expect(jar.get('nm_theme')).toBe('light');
	});

	it('saves the time zone, one Intl knows', async () => {
		const api = backend(() => Response.json({ timezone: 'Asia/Makassar' }));
		const { cookies } = cookieJar();
		expect(await savePreferences({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, { timezone: 'Asia/Makassar' })).toEqual({ ok: true });
		expect(api.calls[0].body).toEqual({ timezone: 'Asia/Makassar' });
		expect(await savePreferences({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, { timezone: 'Mars/Olympus' })).toEqual({ ok: false, status: 400 });
		expect(api.calls).toHaveLength(1);
	});

	it('saves the language and makes the next page use it', async () => {
		const api = backend(() => Response.json({ language: 'id' }));
		const { jar, cookies } = cookieJar();
		expect(await savePreferences({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, { language: 'id' })).toEqual({ ok: true });
		expect(jar.get('PARAGLIDE_LOCALE')).toBe('id');
	});

	it('refuses what the app doesn’t offer, without calling the backend', async () => {
		const api = backend(() => Response.json({}));
		const event = { fetch: api.fetch, cookies: cookieJar().cookies, locals: { accessToken: 'a1' } };
		expect(await savePreferences(event, { theme: 'sepia' })).toEqual({ ok: false, status: 400 });
		expect(await savePreferences(event, { language: 'fr' })).toEqual({ ok: false, status: 400 });
		expect(await savePreferences(event, {})).toEqual({ ok: false, status: 400 });
		expect(api.calls).toEqual([]);
	});

	it('keeps the device as it was when the backend says no', async () => {
		const api = backend(() => new Response('theme must be...', { status: 400 }));
		const { jar, cookies } = cookieJar({ nm_theme: 'dark' });
		expect(await savePreferences({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, { theme: 'light' })).toEqual({ ok: false, status: 400 });
		expect(jar.get('nm_theme')).toBe('dark');
	});
});

describe('chooseProjectsWorkspace', () => {
	it('picks the workspace projects are listed from', async () => {
		const api = backend(() => new Response(null, { status: 204 }));
		expect(await chooseProjectsWorkspace({ fetch: api.fetch, cookies: cookieJar().cookies, locals: { accessToken: 'a1' } }, 'o2')).toEqual({ ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', path: '/api/preferences/project-org', body: { org_id: 'o2' } });
	});

	it('says what went wrong', async () => {
		const api = backend(() => Response.json({ error: 'not a member' }, { status: 403 }));
		expect(await chooseProjectsWorkspace({ fetch: api.fetch, cookies: cookieJar().cookies, locals: { accessToken: 'a1' } }, 'o9')).toEqual({ ok: false, status: 403, message: 'not a member' });
	});
});

describe('createWorkspace', () => {
	it('creates it with the person as its owner', async () => {
		const org = { id: 'o3', name: 'Kopi Kita', role: 'owner', members: 1, projects: 0 };
		const api = backend(() => Response.json(org, { status: 201 }));
		expect(await createWorkspace({ fetch: api.fetch, cookies: cookieJar().cookies, locals: { accessToken: 'a1' } }, '  Kopi Kita ')).toEqual({ ok: true, org });
		expect(api.calls[0]).toMatchObject({ method: 'POST', path: '/api/orgs', body: { name: 'Kopi Kita' } });
	});

	it('asks for a name, and one that fits, before calling the backend', async () => {
		const api = backend(() => Response.json({}));
		const event = { fetch: api.fetch, cookies: cookieJar().cookies, locals: { accessToken: 'a1' } };
		expect(await createWorkspace(event, '   ')).toEqual({ ok: false, status: 400, field: 'required' });
		expect(await createWorkspace(event, 'x'.repeat(81))).toEqual({ ok: false, status: 400, field: 'long' });
		expect(api.calls).toEqual([]);
	});

	it('passes the backend’s own reason on', async () => {
		const api = backend(() => new Response('name is too long', { status: 400 }));
		expect(await createWorkspace({ fetch: api.fetch, cookies: cookieJar().cookies, locals: { accessToken: 'a1' } }, 'Kopi')).toEqual({ ok: false, status: 400, message: 'name is too long' });
	});
});
