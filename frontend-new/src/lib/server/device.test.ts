import { describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { ensureDeviceId, registerDevice, REGISTERED_COOKIE } = await import('./device');
const { APP_VERSION } = await import('$lib/version');

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

const DEVICE = '0f3c2a1e-5b6d-4c7e-8f90-a1b2c3d4e5f6';
const MAC_CHROME = 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36';

describe('ensureDeviceId', () => {
	it("keeps this browser's id once it has one", () => {
		const { jar, cookies } = cookieJar({ device_id: DEVICE });
		expect(ensureDeviceId(cookies)).toBe(DEVICE);
		expect(jar.get('device_id')).toBe(DEVICE);
	});

	it('gives a new browser (or a mangled cookie) an id for good, out of reach of page scripts', () => {
		for (const initial of [{}, { device_id: 'not-a-uuid' }] as Record<string, string>[]) {
			const { jar, options, cookies } = cookieJar(initial);
			const id = ensureDeviceId(cookies);
			expect(id).toMatch(/^[0-9a-f-]{36}$/);
			expect(jar.get('device_id')).toBe(id);
			expect(options.get('device_id')).toMatchObject({ httpOnly: true, path: '/', sameSite: 'lax', maxAge: 2 * 365 * 24 * 60 * 60 });
		}
	});
});

describe('registerDevice', () => {
	function backend() {
		const calls: { url: string; body: unknown }[] = [];
		const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
			calls.push({ url: String(input), body: init?.body ? JSON.parse(String(init.body)) : undefined });
			return new Response(null, { status: 204 });
		});
		return { calls, fetch: fetchFn as unknown as typeof fetch };
	}

	it('puts this browser in the person’s device list, then not again for this version today', async () => {
		const api = backend();
		const { jar, cookies } = cookieJar({ device_id: DEVICE });
		const event = { fetch: api.fetch, cookies, locals: { accessToken: 'a1' } };
		await registerDevice(event, MAC_CHROME);
		await registerDevice(event, MAC_CHROME);
		expect(api.calls).toEqual([
			{
				url: 'http://api.test/api/devices',
				body: { device_id: DEVICE, platform: 'web', app_version: APP_VERSION, name: 'Chrome on macOS', brand: 'Google', model: 'Chrome 130', os_name: 'macOS', os_version: '10.15.7' },
			},
		]);
		expect(jar.get(REGISTERED_COOKIE)).toBe(APP_VERSION);
	});

	it('skips a browser without an id, and a signed-out visitor', async () => {
		const api = backend();
		await registerDevice({ fetch: api.fetch, cookies: cookieJar({}).cookies, locals: { accessToken: 'a1' } }, MAC_CHROME);
		await registerDevice({ fetch: api.fetch, cookies: cookieJar({ device_id: DEVICE }).cookies, locals: {} }, MAC_CHROME);
		expect(api.calls).toEqual([]);
	});
});
