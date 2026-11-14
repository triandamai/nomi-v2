import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');
const preview = await import('./preview/[...path]/+server');
const app = await import('./app/+page.server');
const bySession = await import('../session/[sessionId]/+page.server');

const at = (answer: Parameters<typeof fakeBackend>[0], params: Record<string, string>) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ url: 'http://localhost/projects/p1', params, fetch: api.fetch, locals: { accessToken: 'token' } }) };
};

describe('a project’s workspace', () => {
	it('is not found when the project isn’t there', async () => {
		expect(await thrown(() => load(at(() => new Response('', { status: 404 }), { id: 'p1' }).event as never))).toMatchObject({ status: 404 });
	});

	it('passes the project’s files through for the preview, index first', async () => {
		const { api, event } = at(() => new Response('<h1>Warung</h1>', { headers: { 'content-type': 'text/html' } }), { id: 'p1', path: '' });
		const response = await preview.GET(event as never);
		expect([response.status, response.headers.get('content-type'), await response.text()]).toEqual([200, 'text/html', '<h1>Warung</h1>']);
		expect(api.calls[0].url).toBe('http://api.test/api/projects/p1/preview');
		const asset = at(() => new Response('body{}', { headers: { 'content-type': 'text/css' } }), { id: 'p1', path: 'css/app.css' });
		await preview.GET(asset.event as never);
		expect(asset.api.calls[0].url).toBe('http://api.test/api/projects/p1/preview/css/app.css');
	});

	it('takes the backend’s links to the workspace', async () => {
		expect(await thrown(() => app.load({ params: { id: 'p1' } } as never))).toMatchObject({ status: 303, location: '/projects/p1' });
		expect(await thrown(() => bySession.load(at(() => json({ id: 'p1' }), { sessionId: 's1' }).event as never))).toMatchObject({ location: '/projects/p1' });
		expect(await thrown(() => bySession.load(at(() => new Response('', { status: 404 }), { sessionId: 's1' }).event as never))).toMatchObject({ location: '/chat/s1' });
	});
});
