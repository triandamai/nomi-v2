import { describe, expect, it, vi } from 'vitest';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { forwardShipyardHook } = await import('./shipyardHook');

describe('forwardShipyardHook', () => {
	it('passes the exact bytes and the signature headers on, since the backend checks the signature over them', async () => {
		let seen: { url: string; headers: Record<string, string>; body: string } | undefined;
		const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
			seen = { url: String(input), headers: Object.fromEntries(new Headers(init?.headers)), body: new TextDecoder().decode(init?.body as ArrayBuffer) };
			return new Response('ignored', { status: 202 });
		}) as unknown as typeof fetch;
		const body = '{"type":"deploy.succeeded",  "id":"e1"}';
		const request = new Request('https://v2.test/hooks/shipyard', {
			method: 'POST',
			body,
			headers: { 'content-type': 'application/json', 'shipyard-signature': 't=1,v1=abc', 'shipyard-event-id': 'e1', 'shipyard-event-type': 'deploy.succeeded', 'shipyard-delivery-id': 'd1', cookie: 'access_token=secret' },
		});
		const response = await forwardShipyardHook(request, fetchFn);
		expect(response.status).toBe(202);
		expect(await response.text()).toBe('');
		expect(seen).toEqual({
			url: 'http://api.test/api/hooks/shipyard',
			headers: { 'content-type': 'application/json', 'shipyard-signature': 't=1,v1=abc', 'shipyard-event-id': 'e1', 'shipyard-event-type': 'deploy.succeeded', 'shipyard-delivery-id': 'd1' },
			body,
		});
	});
});
