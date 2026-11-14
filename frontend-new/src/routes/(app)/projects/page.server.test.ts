import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const exportRoute = await import('./[id]/export/+server');

const at = (answer: Parameters<typeof fakeBackend>[0], form?: Record<string, string>, params: Record<string, string> = {}) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ url: 'http://localhost/projects', params, form, fetch: api.fetch, locals: { accessToken: 'token' } }) };
};

describe('Projects', () => {
	it('lists the workspace’s projects and says how many, once the shell has chosen a workspace', async () => {
		const order: string[] = [];
		const { event } = at((call) => (order.push(call.url), call.url.endsWith('/api/projects') ? json({ org_id: null, projects: [{ id: 'a' }, { id: 'b' }] }) : json({})));
		const parent = vi.fn(async () => void order.push('parent'));
		expect(await load({ ...event, parent } as never)).toMatchObject({ title: 'Projects', meta: '2 projects', members: [] });
		expect(order[0]).toBe('parent');
	});

	it('shows the error page when the list can’t be read', async () => {
		expect(await thrown(() => load({ ...at(() => new Response('', { status: 500 })).event, parent: async () => ({}) } as never))).toMatchObject({ status: 503 });
	});

	it('starts a project and opens its workspace', async () => {
		const { api, event } = at((call) => (call.url.endsWith('/api/projects') ? json({ session_id: 's1', project_id: 'p1' }) : json({})), { description: 'Menu page', org: 'w1' });
		expect(await thrown(() => actions.start(event as never))).toMatchObject({ status: 303, location: '/projects/p1' });
		expect(api.calls[0].body).toEqual({ org_id: 'w1' });
	});

	it('asks for a description, and for a workspace when there’s none', async () => {
		expect(await actions.start(at(() => json({}), { description: ' ' }).event as never)).toMatchObject({ status: 400 });
		expect(await actions.start(at(() => new Response('no org', { status: 409 }), { description: 'x' }).event as never)).toMatchObject({ status: 409, data: { needsWorkspace: true } });
	});

	it('deletes a project', async () => {
		expect(await actions.delete(at(() => new Response(null, { status: 204 }), { id: 'p1' }).event as never)).toEqual({ deleted: 'p1' });
	});

	it('passes the export zip through, with its file name', async () => {
		const { event } = at(() => new Response('PK', { headers: { 'content-type': 'application/zip', 'content-disposition': 'attachment; filename="warung.zip"' } }), undefined, { id: 'p1' });
		const response = await exportRoute.GET(event as never);
		expect([response.headers.get('content-disposition'), await response.text()]).toEqual(['attachment; filename="warung.zip"', 'PK']);
	});
});
