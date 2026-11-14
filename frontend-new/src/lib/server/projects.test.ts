import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const projects = await import('./projects');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('projects on the server', () => {
	it('loads the projects and who’s in their workspace', async () => {
		const { event } = eventWith((call) =>
			call.url.endsWith('/api/projects') ? json({ org_id: 'w1', projects: [{ id: 'p1' }], base_domain: 'trian.space' }) : json({ members: [{ user_id: 'u1', name: 'Mario', role: 'owner' }] }),
		);
		expect(await projects.loadProjects(event)).toEqual({ list: { org_id: 'w1', projects: [{ id: 'p1' }], base_domain: 'trian.space' }, members: [{ user_id: 'u1', name: 'Mario', role: 'owner' }] });
	});

	it('has no list when the backend can’t give one, and no members before a workspace is chosen', async () => {
		expect((await projects.loadProjects(eventWith(() => new Response('', { status: 500 })).event)).list).toBeNull();
		const { backend, event } = eventWith(() => json({ org_id: null, projects: [] }));
		expect(await projects.loadProjects(event)).toEqual({ list: { org_id: null, projects: [] }, members: [] });
		expect(backend.calls).toHaveLength(1);
	});

	it('starts a project in a workspace and sends its first message', async () => {
		const { backend, event } = eventWith((call) => (call.url.endsWith('/api/projects') ? json({ session_id: 's1', project_id: 'p1' }) : json({ user_message: {} })));
		expect(await projects.startProject(event, 'w1', 'A menu page')).toEqual({ ok: true, sessionId: 's1', projectId: 'p1', sent: true });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body])).toEqual([
			['POST', '/api/projects', { org_id: 'w1' }],
			['POST', '/api/sessions/s1/messages', { text: 'A menu page' }],
		]);
	});

	it('says when a workspace has to be chosen first', async () => {
		const { event } = eventWith(() => new Response('pick an organization', { status: 409 }));
		expect(await projects.startProject(event, null, 'x')).toEqual({ ok: false, status: 409, needsWorkspace: true, message: 'pick an organization' });
	});

	it('deletes a project', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		expect(await projects.deleteProject(event, 'p 1')).toEqual({ ok: true });
		expect(backend.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/projects/p%201' });
	});
});
