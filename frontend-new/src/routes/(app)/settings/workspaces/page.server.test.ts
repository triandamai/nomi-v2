import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const event = (answer: Parameters<typeof fakeBackend>[0], form?: Record<string, string>) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ url: 'http://localhost/settings/workspaces', form, fetch: api.fetch, locals: { accessToken: 'token' } }) };
};
const org = { id: 'a', name: 'Kopi Kita', role: 'owner', members: 2, projects: 1 };

describe('Settings › Workspaces', () => {
	it('lists the workspaces and says how many', async () => {
		const { event: e } = event(() => json({ orgs: [org, { ...org, id: 'b' }], project_org_id: 'a' }));
		expect(await load(e as never)).toMatchObject({ title: 'Workspaces', meta: '2 workspaces', list: { project_org_id: 'a' } });
	});

	it('shows the error page when they can’t be read', async () => {
		const { event: e } = event(() => new Response('', { status: 502 }));
		expect(await thrown(() => load(e as never))).toMatchObject({ status: 503 });
	});

	it('creates one, or says what’s wrong with the name', async () => {
		const made = event(() => json(org, 201), { name: ' Kopi Kita ' });
		expect(await actions.create(made.event as never)).toEqual({ created: org });
		expect(made.api.calls[0].body).toEqual({ name: 'Kopi Kita' });
		expect(await actions.create(event(() => json(org, 201), { name: '  ' }).event as never)).toMatchObject({ status: 400, data: { field: 'name', message: 'Give the workspace a name.' } });
	});

	it('chooses where projects come from', async () => {
		const used = event(() => new Response(null, { status: 204 }), { id: 'b' });
		expect(await actions.use(used.event as never)).toEqual({ used: 'b' });
		expect(used.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/preferences/project-org', body: { org_id: 'b' } });
	});
});
