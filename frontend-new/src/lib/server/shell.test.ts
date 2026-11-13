import { describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { loadShell } = await import('./shell');

function cookieJar(initial: Record<string, string> = {}) {
	const jar = new Map(Object.entries(initial));
	const cookies = {
		get: (name: string) => jar.get(name),
		set: (name: string, value: string) => void jar.set(name, value),
		delete: (name: string) => void jar.delete(name),
	} as unknown as Cookies;
	return { jar, cookies };
}

type Route = { status?: number; body?: unknown };

/** A fake backend answering each path (method + path) with its route, 404 otherwise. */
function backend(routes: Record<string, Route>) {
	const calls: { method: string; path: string; body?: unknown }[] = [];
	const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
		const url = new URL(String(input));
		const method = init?.method ?? 'GET';
		calls.push({ method, path: url.pathname + url.search, ...(init?.body ? { body: JSON.parse(String(init.body)) } : {}) });
		const route = routes[`${method} ${url.pathname}${url.search}`];
		if (!route) return new Response(null, { status: 404 });
		return route.body === undefined ? new Response(null, { status: route.status ?? 204 }) : Response.json(route.body, { status: route.status ?? 200 });
	});
	return { calls, fetch: fetchFn as unknown as typeof fetch };
}

const healthy: Record<string, Route> = {
	'GET /api/profile': { body: { display_name: 'Mario Prasetyo', username: 'mario', email: 'mario@nomi.test', avatar_url: null } },
	'GET /api/preferences': { body: { theme: 'dark', timezone: 'Asia/Jakarta', language: 'en', has_stored_timezone: true } },
	'GET /api/agents': {
		body: {
			agents: [
				{ agent_type: 'chitchat', name: 'Nomi', role: 'Talks with you and routes the work', is_dynamic: false, state: 'idle', status: '' },
				{ agent_type: 'money', name: 'Dana', role: 'Transactions, budgets, subscriptions', is_dynamic: false, state: 'idle', status: '' },
			],
		},
	},
	'GET /api/usage/brief': { body: { plan: { id: 'free', name: 'Free', monthly_tokens: 1_000_000, custom_quota: false }, month: '2026-10', tokens_used: 21_800, can_upgrade: true } },
	'GET /api/notifications/unread': { body: { unread: 3, latest: null } },
	'GET /api/sessions': {
		body: {
			sessions: [
				{ id: 's1', title: 'Budget belanja Oktober', last_message: null, updated_at: '2026-10-21T06:00:00Z', project_id: null, pinned: false, in_project_id: null },
				{ id: 's2', title: 'salah-daily-tracker', last_message: null, updated_at: '2026-10-19T08:00:00Z', project_id: 'p1', pinned: false, in_project_id: null },
			],
		},
	},
	'GET /api/orgs': {
		body: {
			orgs: [
				{ id: 'o1', name: 'Mario', role: 'owner', members: 1, projects: 1 },
				{ id: 'o2', name: "Trian's space", role: 'admin', members: 3, projects: 4 },
			],
			project_org_id: 'o2',
		},
	},
	'POST /api/devices': { status: 204 },
};

const now = new Date('2026-10-21T08:14:30Z');

function event(routes: Record<string, Route>, cookies: Record<string, string> = {}) {
	const api = backend(routes);
	const jar = cookieJar({ device_id: '7d1f0c2e-3b0a-4c55-9a5e-1f2a3b4c5d6e', ...cookies });
	return { api, jar, event: { fetch: api.fetch, cookies: jar.cookies, locals: { accessToken: 'a1' }, userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 14_5) Chrome/131.0' } };
}

describe('loadShell', () => {
	it('loads the person, their crew, usage, inbox, chats and workspaces, as the shell shows them', async () => {
		const { event: e } = event(healthy);
		const shell = await loadShell(e, now);
		expect(shell.person).toEqual({ name: 'Mario Prasetyo', firstName: 'Mario', avatar: null });
		expect(shell.theme).toBe('dark');
		expect(shell.timeZone).toBe('Asia/Jakarta');
		expect(shell.usage).toEqual({ card: { plan: 'Free plan', used: 0.0218, caption: '21.8K / 1M tokens this month' }, account: 'Free · 21.8K / 1M tokens', canUpgrade: true });
		expect(shell.unread).toBe(3);
		expect(shell.chats).toBe(1);
		expect(shell.recents.map((recent) => recent.href)).toEqual(['/chat/s1', '/projects/p1']);
		expect(shell.crew.rows.map((row) => row.name)).toEqual(['Nomi', 'Dana']);
		expect(shell.crew.summary).toEqual({ kind: 'idle', label: '2 idle' });
		expect(shell.workspaces.map((row) => [row.name, row.chosen])).toEqual([
			['Mario', false],
			["Trian's space", true],
		]);
		expect(shell.projectsWorkspace).toEqual({ id: 'o2', name: "Trian's space", projects: 4 });
		expect(shell.deviceId).toBe('7d1f0c2e-3b0a-4c55-9a5e-1f2a3b4c5d6e');
		expect(shell.now).toBe(now.toISOString());
	});

	it('loads it all at once, and puts this browser in the device list', async () => {
		const { event: e, api } = event(healthy);
		await loadShell(e, now);
		expect(api.calls.map((call) => `${call.method} ${call.path}`).sort()).toEqual(
			['GET /api/agents', 'GET /api/notifications/unread', 'GET /api/orgs', 'GET /api/preferences', 'GET /api/profile', 'GET /api/sessions', 'GET /api/usage/brief', 'POST /api/devices'].sort(),
		);
	});

	it('keeps the saved theme and language on this device for the next page', async () => {
		const { event: e, jar } = event({ ...healthy, 'GET /api/preferences': { body: { theme: 'light', timezone: 'UTC', language: 'id', has_stored_timezone: false } } });
		const shell = await loadShell(e, now);
		expect(jar.jar.get('nm_theme')).toBe('light');
		expect(jar.jar.get('PARAGLIDE_LOCALE')).toBe('id');
		// No zone saved: the browser's own decides the greeting.
		expect(shell.timeZone).toBeNull();
	});

	it('lists projects from the person’s first workspace until they pick one', async () => {
		const routes = { ...healthy, 'GET /api/orgs': { body: { orgs: [{ id: 'o1', name: 'E2E space', role: 'owner', members: 1, projects: 0 }], project_org_id: null } }, 'PUT /api/preferences/project-org': { status: 204 } };
		const { event: e, api } = event(routes);
		const shell = await loadShell(e, now);
		expect(api.calls).toContainEqual({ method: 'PUT', path: '/api/preferences/project-org', body: { org_id: 'o1' } });
		expect(shell.projectsWorkspace).toEqual({ id: 'o1', name: 'E2E space', projects: 0 });
		expect(shell.workspaces[0].chosen).toBe(true);
	});

	it('still opens when parts of it can’t be loaded', async () => {
		const failing = Object.fromEntries(Object.keys(healthy).map((key) => [key, { status: 500, body: { error: 'down' } }]));
		const { event: e } = event(failing);
		const shell = await loadShell(e, now);
		expect(shell.person).toEqual({ name: 'You', firstName: 'You', avatar: null });
		expect(shell.theme).toBe('system');
		expect(shell.usage).toBeNull();
		expect(shell.unread).toBe(0);
		expect(shell.recents).toEqual([]);
		expect(shell.crew.rows).toEqual([]);
		expect(shell.workspaces).toEqual([]);
		expect(shell.projectsWorkspace).toBeNull();
	});
});
