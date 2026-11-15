import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const exportRoute = await import('./[id]/export/+server');
const aboutRoute = await import('./[id]/about/+server');
const moveRoute = await import('./[id]/move/+server');

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

	it('says the list couldn’t be read, with the status, and leaves the count out', async () => {
		const loaded = await load({ ...at(() => new Response('', { status: 500 })).event, parent: async () => ({}) } as never);
		expect(loaded).toMatchObject({ title: 'Projects', list: null, failure: { status: 500 } });
		expect(loaded).not.toHaveProperty('meta');
	});

	it('stops Koda, asks it to carry on, and passes on a change asked from the panel, in the project’s chat', async () => {
		const sent = (action: 'stop' | 'resume' | 'ask', form: Record<string, string>) => {
			const { api, event } = at(() => json({ user_message: {} }), form);
			return Promise.resolve(actions[action](event as never)).then((result: unknown) => ({ result, call: api.calls[0] }));
		};
		expect(await sent('stop', { session: 's1' })).toMatchObject({ result: { stopped: 's1' }, call: { url: 'http://api.test/api/sessions/s1/messages', body: { text: 'stop' } } });
		expect(await sent('resume', { session: 's1' })).toMatchObject({ result: { resumed: 's1' }, call: { body: { text: 'Carry on building where you stopped.' } } });
		expect(await sent('ask', { session: 's1', text: 'Add a Maghrib reminder' })).toMatchObject({ result: { asked: 's1' }, call: { body: { text: 'Add a Maghrib reminder' } } });
		expect(await actions.ask(at(() => json({}), { session: 's1', text: ' ' }).event as never)).toMatchObject({ status: 400 });
	});

	it('asks Koda to fix a failed deploy or check an app that’s down, in the project’s chat', async () => {
		const sent = (action: 'fix' | 'check') => {
			const { api, event } = at(() => json({ user_message: {} }), { session: 's1' });
			return Promise.resolve(actions[action](event as never)).then((result: unknown) => ({ result, call: api.calls[0] }));
		};
		expect(await sent('fix')).toMatchObject({ result: { asked: 's1' }, call: { url: 'http://api.test/api/sessions/s1/messages', body: { text: 'The last deploy failed. Please look at why and fix it.' } } });
		expect(await sent('check')).toMatchObject({ result: { asked: 's1' }, call: { body: { text: 'The live app is down. Please check why and get it back up.' } } });
		expect(await actions.fix(at(() => json({}), {}).event as never)).toMatchObject({ status: 400 });
	});

	it('renames a project and makes a copy of one', async () => {
		const renamed = at(() => json({ id: 'p1' }), { id: 'p1', name: ' kopi-menu ' });
		expect(await actions.rename(renamed.event as never)).toEqual({ renamed: 'p1' });
		expect(renamed.api.calls[0]).toMatchObject({ method: 'PATCH', body: { name: 'kopi-menu' } });
		expect(await actions.rename(at(() => json({}), { id: 'p1', name: ' ' }).event as never)).toMatchObject({ status: 400 });
		expect(await actions.rename(at(() => new Response('that name is taken', { status: 409 }), { id: 'p1', name: 'x' }).event as never)).toMatchObject({ status: 409, data: { message: 'that name is taken' } });
		expect(await actions.duplicate(at(() => json({ session_id: 's2', project_id: 'p2' }, 201), { id: 'p1' }).event as never)).toEqual({ duplicated: 'p2' });
	});

	it('deploys a draft, and says why when it can’t', async () => {
		const { api, event } = at(() => json({}), { id: 'p1' });
		expect(await actions.deploy(event as never)).toEqual({ deploying: 'p1' });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/deploy/publish' });
		expect(await actions.deploy(at(() => new Response('publishing is not configured', { status: 503 }), { id: 'p1' }).event as never)).toMatchObject({ status: 503, data: { message: 'publishing is not configured' } });
	});

	it('moves a project to the workspace picked, its name as the confirmation', async () => {
		const { api, event } = at(() => new Response(null, { status: 204 }), { id: 'p1', target: 'w2', name: 'warung' });
		expect(await actions.move(event as never)).toEqual({ moved: 'p1' });
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/transfer', body: { target_org_id: 'w2', confirm_name: 'warung' } });
		expect(await actions.move(at(() => json({}), { id: 'p1', target: '' }).event as never)).toMatchObject({ status: 400 });
	});

	it('answers the detail panel and the Move dialog', async () => {
		const about = await aboutRoute.GET(at(() => new Response('', { status: 404 }), undefined, { id: 'p1' }).event as never);
		expect(about.status).toBe(404);
		const move = await moveRoute.GET(at(() => new Response('', { status: 403 }), undefined, { id: 'p1' }).event as never);
		expect(move.status).toBe(503);
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
