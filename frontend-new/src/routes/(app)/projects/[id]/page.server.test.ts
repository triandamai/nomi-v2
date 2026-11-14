import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const deploy = await import('./deploy/+server');
const domainCheck = await import('./domain/check/+server');
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

	it('publishes and rolls back, and says why not', async () => {
		const ok = at(() => json({}, 202), { id: 'p1' });
		expect(await actions.publish(ok.event as never)).toEqual({ publishing: true });
		expect(ok.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/deploy/publish' });
		expect(await actions.publish(at(() => new Response("there's nothing to publish yet", { status: 422 }), { id: 'p1' }).event as never)).toMatchObject({ status: 422, data: { message: "there's nothing to publish yet" } });
		const back = at(() => json({}, 202), { id: 'p1' });
		const withForm = { ...back.event, request: new Request('http://localhost/projects/p1', { method: 'POST', body: new URLSearchParams({ publish: 'pub1' }) }) };
		expect(await actions.rollback(withForm as never)).toEqual({ rolledBack: 'pub1' });
		expect(back.api.calls[0].url).toBe('http://api.test/api/projects/p1/deploy/publishes/pub1/rollback');
	});

	it('gives the deploy overview to look at again', async () => {
		const response = await deploy.GET(at(() => json({ available: false, publishes: [] }), { id: 'p1' }).event as never);
		expect(await response.json()).toEqual({ available: false, publishes: [] });
	});

	it('changes the address, and passes on why not with free names', async () => {
		const form = (body: Record<string, string>, answer: Parameters<typeof fakeBackend>[0]) => {
			const { api, event } = at(answer, { id: 'p1' });
			return { api, event: { ...event, request: new Request('http://localhost/projects/p1', { method: 'POST', body: new URLSearchParams(body) }) } };
		};
		const done = form({ name: ' Kopi ' }, () => json({ url: 'https://kopi.trian.space' }));
		expect(await actions.domain(done.event as never)).toEqual({ url: 'https://kopi.trian.space' });
		expect(done.api.calls[0].body).toEqual({ subdomain: 'kopi' });
		expect(await actions.domain(form({ name: 'kopi' }, () => json({ available: false, message: 'taken', suggestions: ['kopi-1'] }, 409)).event as never)).toMatchObject({ status: 409, data: { message: 'taken', suggestions: ['kopi-1'] } });
		expect(await actions.domain(form({ name: '' }, () => json({})).event as never)).toMatchObject({ status: 400 });
	});

	it('checks a name for the field', async () => {
		const { api, event } = at(() => json({ available: true }), { id: 'p1' });
		const response = await domainCheck.GET({ ...event, url: new URL('http://localhost/projects/p1/domain/check?name=kopi') } as never);
		expect(await response.json()).toEqual({ available: true });
		expect(api.calls[0].url).toBe('http://api.test/api/projects/p1/domain/check?subdomain=kopi');
	});
});
