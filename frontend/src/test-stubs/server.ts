// Test helpers for `+page.server.ts` loads and actions: a fake request event whose form and URL the
// test chooses, and a recorder for the backend calls `apiFetch` makes.
import type { Cookies } from '@sveltejs/kit';
import { vi } from 'vitest';

export const json = (body: unknown, status = 200) =>
	new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

export const textResponse = (body: string, status: number) => new Response(body, { status });

export function cookieJar(values: Record<string, string> = {}): Cookies {
	const jar = new Map(Object.entries(values));
	return {
		get: (name: string) => jar.get(name),
		getAll: () => [...jar].map(([name, value]) => ({ name, value })),
		set: vi.fn((name: string, value: string) => void jar.set(name, value)),
		delete: vi.fn((name: string) => void jar.delete(name)),
		serialize: () => '',
	} as unknown as Cookies;
}

export function event(
	options: { form?: Record<string, string | string[]>; url?: string; params?: Record<string, string>; cookies?: Cookies; locals?: unknown } = {},
) {
	const url = new URL(options.url ?? 'http://localhost/');
	let request: Request;
	if (options.form) {
		const body = new FormData();
		for (const [name, value] of Object.entries(options.form)) {
			for (const v of Array.isArray(value) ? value : [value]) body.append(name, v);
		}
		request = new Request(url, { method: 'POST', body });
	} else {
		request = new Request(url);
	}
	return {
		request,
		url,
		params: options.params ?? {},
		cookies: options.cookies ?? cookieJar(),
		fetch,
		locals: options.locals ?? {},
		setHeaders: () => {},
		parent: async () => ({}),
		depends: () => {},
		untrack: <T>(fn: () => T) => fn(),
		getClientAddress: () => '127.0.0.1',
		platform: undefined,
		route: { id: '' },
		isDataRequest: false,
		isSubRequest: false,
		isRemoteRequest: false,
	} as never;
}

/** Calls an action or load that may `throw redirect(...)`/`error(...)`, returning what it threw instead. */
export async function settle<T>(fn: () => Promise<T> | T): Promise<T | { thrown: unknown }> {
	try {
		return await fn();
	} catch (thrown) {
		return { thrown };
	}
}
