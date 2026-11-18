import { describe, expect, it, vi } from 'vitest';
import { cookieJar } from '../../../../../test-stubs/server';

vi.mock('$lib/server/api', () => ({
	apiFetch: vi.fn(async (_fetch: unknown, _cookies: unknown, path: string, _init?: RequestInit) => {
		return new Response(JSON.stringify({ path }), {
			status: 200,
			headers: { 'content-type': 'application/json' },
		});
	}),
}));

const { GET, PUT, POST } = await import('./+server');

function makeRequest(handler: typeof GET, rest: string, method = 'GET', body?: string) {
	const cookies = cookieJar({ access_token: 'valid' });
	const fetch = vi.fn();
	const request = new Request(`http://localhost/admin/api/${rest}`, {
		method,
		body,
	});
	return handler({
		params: { rest },
		request,
		url: new URL(`http://localhost/admin/api/${rest}`),
		cookies,
		fetch,
	} as never);
}

describe('admin api proxy [...rest]', () => {
	it('proxies payment overview and provider endpoints', async () => {
		const overview = await makeRequest(GET, 'payment');
		expect(overview.status).toBe(200);
		expect(await overview.json()).toEqual({ path: '/api/admin/payment' });

		const choose = await makeRequest(PUT, 'payment', 'PUT', JSON.stringify({ provider: 'xendit' }));
		expect(choose.status).toBe(200);
		expect(await choose.json()).toEqual({ path: '/api/admin/payment' });

		const provider = await makeRequest(GET, 'payment/xendit');
		expect(provider.status).toBe(200);
		expect(await provider.json()).toEqual({ path: '/api/admin/payment/xendit' });

		const test = await makeRequest(POST, 'payment/doku/test', 'POST');
		expect(test.status).toBe(200);
		expect(await test.json()).toEqual({ path: '/api/admin/payment/doku/test' });
	});

	it('refuses paths not on the allowlist with a 404', async () => {
		await expect(makeRequest(GET, 'forbidden/endpoint')).rejects.toMatchObject({ status: 404 });
	});
});
