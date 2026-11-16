import { describe, expect, it, vi } from 'vitest';
import { cookieJar } from '../../../../test-stubs/server';

vi.mock('$lib/server/api', () => ({ apiUrl: (path: string) => `http://api.test${path}` }));

const { GET } = await import('./+server');

const TOKEN = 'ab'.repeat(32);

function request(path: string, answer: Response) {
	const fetch = vi.fn(async (_url: string, _init?: RequestInit) => answer);
	const cookies = cookieJar({ access_token: 'secret', refresh_token: 'also-secret' });
	const call = GET({ params: { token: TOKEN, path }, fetch, cookies, url: new URL(`http://localhost/design-preview/${TOKEN}/${path}`) } as never);
	return { call: call as Promise<Response>, fetch };
}

const page = () =>
	new Response('<html>Hi</html>', {
		status: 200,
		headers: {
			'content-type': 'text/html; charset=utf-8',
			'content-security-policy': "sandbox allow-scripts; connect-src 'none'",
			'referrer-policy': 'no-referrer',
			'x-content-type-options': 'nosniff',
			'cache-control': 'private, max-age=3600',
			'access-control-allow-origin': '*',
			'cross-origin-resource-policy': 'cross-origin',
			'x-robots-tag': 'noindex, nofollow',
			'set-cookie': 'sneaky=1',
		},
	});

describe('design preview files', () => {
	it('fetches the file by its token alone, with no sign-in', async () => {
		const { call, fetch } = request('menu/index.html', page());
		const response = await call;
		expect(fetch).toHaveBeenCalledTimes(1);
		const [url, init] = fetch.mock.calls[0];
		expect(url).toBe(`http://api.test/api/design/preview/${TOKEN}/menu/index.html`);
		expect(JSON.stringify(init ?? {})).not.toContain('secret');
		expect(await response.text()).toBe('<html>Hi</html>');
	});

	it('keeps what fences the page in, and nothing else', async () => {
		const response = await request('', page()).call;
		expect(response.status).toBe(200);
		expect(response.headers.get('content-type')).toBe('text/html; charset=utf-8');
		expect(response.headers.get('content-security-policy')).toBe("sandbox allow-scripts; connect-src 'none'");
		expect(response.headers.get('referrer-policy')).toBe('no-referrer');
		expect(response.headers.get('x-content-type-options')).toBe('nosniff');
		expect(response.headers.get('cache-control')).toBe('private, max-age=3600');
		expect(response.headers.get('cross-origin-resource-policy')).toBe('cross-origin');
		expect(response.headers.get('x-robots-tag')).toBe('noindex, nofollow');
		expect(response.headers.get('set-cookie')).toBeNull();
	});

	it('fences the page in even when the answer forgot to', async () => {
		const response = await request('index.html', new Response('<html>', { status: 200, headers: { 'content-type': 'text/html' } })).call;
		expect(response.headers.get('content-security-policy')).toMatch(/^sandbox allow-scripts/);
		expect(response.headers.get('content-security-policy')).not.toContain('allow-same-origin');
		expect(response.headers.get('content-security-policy')).toContain("frame-ancestors 'self'");
		expect(response.headers.get('x-robots-tag')).toBe('noindex, nofollow');
	});

	it('passes a missing file on as missing', async () => {
		const response = await request('nope.html', new Response('not found', { status: 404 })).call;
		expect(response.status).toBe(404);
	});

	it('asks for nothing when the name is not a token', async () => {
		const fetch = vi.fn();
		const response = (await GET({ params: { token: '../../api/profile', path: '' }, fetch, cookies: cookieJar(), url: new URL('http://localhost/') } as never)) as Response;
		expect(response.status).toBe(404);
		expect(fetch).not.toHaveBeenCalled();
	});
});
