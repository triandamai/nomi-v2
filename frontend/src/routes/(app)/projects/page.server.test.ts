import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, settle } from '../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => ({ path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(init.body) : undefined }));
const status = (result: unknown) => (result as { status?: number }).status;

function backend(override: (path: string, method: string) => Response | undefined = () => undefined) {
	apiFetch.mockImplementation(async (_f: unknown, _c: unknown, path: string, init?: RequestInit) => {
		const custom = override(path, init?.method ?? 'GET');
		if (custom) return custom;
		if (path === '/api/orgs') return json({ orgs: [{ id: 'o1', name: 'Acme' }], project_org_id: 'o1' });
		if (path === '/api/projects') return json({ org_id: 'o1', projects: [{ id: 'p1', name: 'Bakery' }] });
		if (path === '/api/sessions') return json({ sessions: [{ id: 's1' }] });
		if (path === '/api/orgs/o1') return json({ members: [{ user_id: 'u1', name: 'Ana', role: 'owner' }] });
		return json({});
	});
}

beforeEach(() => {
	apiFetch.mockReset();
	backend();
});

describe('projects: load', () => {
	it('loads organizations, projects, chats and the organization’s members', async () => {
		expect(await load(event())).toEqual({
			orgs: [{ id: 'o1', name: 'Acme' }],
			orgId: 'o1',
			projects: [{ id: 'p1', name: 'Bakery' }],
			sessions: [{ id: 's1' }],
			members: [{ user_id: 'u1', name: 'Ana', role: 'owner' }],
		});
	});

	it('still loads without chats or members, and fails without projects', async () => {
		backend((path) => (path === '/api/sessions' || path === '/api/orgs/o1' ? json({}, 500) : undefined));
		expect(await load(event())).toMatchObject({ sessions: [], members: [] });
		backend((path) => (path === '/api/projects' ? json({}, 500) : undefined));
		expect(await settle(() => load(event()))).toMatchObject({ thrown: { status: 500 } });
	});
});

describe('projects: actions', () => {
	it('picks or creates the organization projects live in', async () => {
		expect(status(await actions.chooseOrg(event({ form: {} })))).toBe(400);
		expect(await actions.chooseOrg(event({ form: { org_id: 'o2' } }))).toEqual({ chosen: true });
		expect(calls().at(-1)).toMatchObject({ path: '/api/preferences/project-org', method: 'PUT', body: { org_id: 'o2' } });
		expect(status(await actions.createOrg(event({ form: { name: ' ' } })))).toBe(400);
		backend((path) => (path === '/api/orgs' ? json({ id: 'o3' }) : undefined));
		expect(await actions.createOrg(event({ form: { name: 'Team' } }))).toEqual({ chosen: true });
		backend(() => json({}, 500));
		expect(status(await actions.chooseOrg(event({ form: { org_id: 'o2' } })))).toBe(500);
		expect(status(await actions.createOrg(event({ form: { name: 'Team' } })))).toBe(500);
	});

	it('starts a project and opens it, with the first prompt', async () => {
		backend((path, method) => (path === '/api/projects' && method === 'POST' ? json({ session_id: 's9', project_id: 'p9' }) : undefined));
		expect(await settle(() => actions.createProject(event({ form: { org_id: 'o1', prompt: 'a bakery site' } })))).toMatchObject({
			thrown: { status: 303, location: '/projects/session/s9?prompt=a%20bakery%20site' },
		});
		expect(await settle(() => actions.createProject(event({ form: {} })))).toMatchObject({ thrown: { location: '/projects/session/s9' } });
		expect(await actions.createForImport(event({ form: { org_id: 'o1' } }))).toEqual({ imported: { session_id: 's9', project_id: 'p9' } });
	});

	it('asks for an organization first, and reports other failures', async () => {
		backend((path, method) => (path === '/api/projects' && method === 'POST' ? json({}, 409) : undefined));
		expect(await actions.createProject(event({ form: {} }))).toMatchObject({ status: 409, data: { needsOrg: true } });
		expect(await actions.createForImport(event({ form: {} }))).toMatchObject({ status: 409 });
		backend((path, method) => (path === '/api/projects' && method === 'POST' ? json({}, 500) : undefined));
		expect(await settle(() => actions.createProject(event({ form: {} })))).toMatchObject({ thrown: { status: 500 } });
		expect(status(await actions.createForImport(event({ form: {} })))).toBe(500);
	});

	it('deletes a project', async () => {
		expect(status(await actions.deleteProject(event({ form: {} })))).toBe(400);
		expect(await actions.deleteProject(event({ form: { project_id: 'p1' } }))).toEqual({ deleted: 'p1' });
		backend(() => json({}, 403));
		expect(status(await actions.deleteProject(event({ form: { project_id: 'p1' } })))).toBe(403);
	});
});
