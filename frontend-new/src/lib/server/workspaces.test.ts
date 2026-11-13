import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const workspaces = await import('./workspaces');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('a workspace on the server', () => {
	it('loads the workspace with its summary, or nothing when the person isn’t in it', async () => {
		const { event } = eventWith((call) =>
			call.url.endsWith('/api/orgs') ? json({ orgs: [{ id: 'w1', name: 'Kopi', role: 'owner', members: 2, projects: 3 }], project_org_id: 'w1' }) : json({ id: 'w1', name: 'Kopi', my_role: 'owner', me: 'u1', members: [], invites: [] }),
		);
		expect(await workspaces.loadWorkspace(event, 'w1')).toMatchObject({ detail: { name: 'Kopi' }, summary: { projects: 3 }, projectsHere: true });
		const out = eventWith(() => new Response('not a member', { status: 403 }));
		expect(await workspaces.loadWorkspace(out.event, 'w1')).toMatchObject({ detail: null, status: 403 });
	});

	it('invites one address at a time, and passes the backend’s reason on', async () => {
		const { backend, event } = eventWith(() => json({ code: 'c1', email: 'a@b.co', role: 'member', created_at: '', expires_at: '' }, 201));
		expect(await workspaces.invite(event, 'w1', 'a@b.co', 'member')).toMatchObject({ ok: true, data: { code: 'c1' } });
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/orgs/w1/invites', body: { email: 'a@b.co', role: 'member' } });
		const taken = eventWith(() => new Response('they\'re already a member', { status: 409 }));
		expect(await workspaces.invite(taken.event, 'w1', 'a@b.co', 'member')).toEqual({ ok: false, status: 409, message: 'they\'re already a member' });
	});

	it('renames, revokes, changes a role and removes, each on its own path', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		await workspaces.rename(event, 'w1', 'Kopi Kita');
		await workspaces.revoke(event, 'w1', 'c 1');
		await workspaces.setRole(event, 'w1', 'u2', 'admin');
		await workspaces.removeMember(event, 'w1', 'u2');
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body ?? null])).toEqual([
			['PATCH', '/api/orgs/w1', { name: 'Kopi Kita' }],
			['DELETE', '/api/orgs/w1/invites/c%201', null],
			['PATCH', '/api/orgs/w1/members/u2', { role: 'admin' }],
			['DELETE', '/api/orgs/w1/members/u2', null],
		]);
	});
});
