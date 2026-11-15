import { describe, expect, it, vi } from 'vitest';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { forwardSiteReport } = await import('./siteHook');

describe('forwardSiteReport', () => {
	it('passes a published site’s report on as it came, answering empty to any site', async () => {
		let seen: { url: string; body: string; headers: Record<string, string> } | undefined;
		const fetchFn = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
			seen = { url: String(input), body: new TextDecoder().decode(init?.body as ArrayBuffer), headers: Object.fromEntries(new Headers(init?.headers)) };
			return new Response(null, { status: 204 });
		}) as unknown as typeof fetch;
		const report = '{"k":"v","r":"https://www.instagram.com/","h":"warungkopi.id"}';
		const request = new Request('https://v2.test/hooks/sites/p1', { method: 'POST', body: report, headers: { cookie: 'access_token=secret', origin: 'https://warungkopi.id' } });
		const response = await forwardSiteReport('p 1', request, fetchFn);
		expect([response.status, response.headers.get('access-control-allow-origin')]).toEqual([204, '*']);
		expect(seen).toEqual({ url: 'http://api.test/api/hooks/sites/p%201', body: report, headers: {} });
	});

	it('drops a report too big to be one, and still answers empty when the backend can’t be reached', async () => {
		const fetchFn = vi.fn(async () => {
			throw new Error('down');
		}) as unknown as typeof fetch;
		const big = new Request('https://v2.test/hooks/sites/p1', { method: 'POST', body: 'x'.repeat(4096) });
		expect((await forwardSiteReport('p1', big, fetchFn)).status).toBe(204);
		expect(fetchFn).not.toHaveBeenCalled();
		const small = new Request('https://v2.test/hooks/sites/p1', { method: 'POST', body: '{}' });
		expect((await forwardSiteReport('p1', small, fetchFn)).status).toBe(204);
	});
});
