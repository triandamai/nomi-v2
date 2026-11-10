import { describe, expect, it, vi } from 'vitest';
import type { Cookies } from '@sveltejs/kit';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { proxyFile } = await import('./fileProxy');

const cookies = { get: () => undefined, set: () => {}, delete: () => {} } as unknown as Cookies;

function backend(answers: Record<string, Response>) {
	const urls: string[] = [];
	const fetchFn = vi.fn(async (input: RequestInfo | URL) => {
		urls.push(String(input));
		return answers[String(input)] ?? new Response(null, { status: 404 });
	});
	return { urls, fetch: fetchFn as unknown as typeof fetch };
}

describe('proxyFile', () => {
	it('streams a file through the API with its type, and keeps the browser from guessing it', async () => {
		const api = backend({ 'http://api.test/api/avatars/u1/a.png': new Response('png', { headers: { 'content-type': 'image/png' } }) });
		const response = await proxyFile({ fetch: api.fetch, cookies, locals: { accessToken: 'a1' } }, '/api/avatars/u1/a.png', { fallbackType: 'application/octet-stream' });
		expect(response.status).toBe(200);
		expect(await response.text()).toBe('png');
		expect(Object.fromEntries(response.headers)).toMatchObject({ 'content-type': 'image/png', 'cache-control': 'private, max-age=3600', 'x-content-type-options': 'nosniff' });
	});

	it('tries the public copy for someone signed out (an avatar on a shared page)', async () => {
		const api = backend({
			'http://api.test/api/avatars/u1/a.png': new Response(null, { status: 401 }),
			'http://api.test/api/public/avatars/u1/a.png': new Response('png', { headers: { 'content-type': 'image/png' } }),
		});
		const response = await proxyFile({ fetch: api.fetch, cookies, locals: {} }, '/api/avatars/u1/a.png', { publicPath: '/api/public/avatars/u1/a.png', fallbackType: 'application/octet-stream' });
		expect(response.status).toBe(200);
		expect(api.urls).toEqual(['http://api.test/api/avatars/u1/a.png', 'http://api.test/api/public/avatars/u1/a.png']);
	});

	it('answers 404 for anything it cannot show, 401 when only signing in would help', async () => {
		const missing = backend({});
		expect((await proxyFile({ fetch: missing.fetch, cookies, locals: { accessToken: 'a1' } }, '/api/voice/u1/x.mp3', { fallbackType: 'audio/mpeg' })).status).toBe(404);
		const locked = backend({ 'http://api.test/api/voice/u1/x.mp3': new Response(null, { status: 401 }) });
		expect((await proxyFile({ fetch: locked.fetch, cookies, locals: {} }, '/api/voice/u1/x.mp3', { fallbackType: 'audio/mpeg' })).status).toBe(401);
	});
});
