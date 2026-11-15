import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const projects = await import('./projects');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('projects on the server', () => {
	it('loads the projects, who’s in their workspace, their chats and whether this Nomi publishes', async () => {
		const list = { org_id: 'w1', projects: [{ id: 'p1', session_id: 's1', files_count: 3 }, { id: 'p2', session_id: 's2', deploy_url: 'https://p2.trian.space' }], base_domain: 'trian.space' };
		const sessions = [
			{ id: 's1', title: 'Build a menu', updated_at: '2026-11-13T09:00:00Z', project_id: 'p1', in_project_id: null, agent_active: true },
			{ id: 'c2', title: 'Ideas', updated_at: '2026-11-12T09:00:00Z', project_id: null, in_project_id: 'p2' },
			{ id: 'c3', title: 'Lunch', updated_at: '2026-11-12T09:00:00Z', project_id: null, in_project_id: null },
		];
		const { backend, event } = eventWith((call) =>
			call.url.endsWith('/api/projects')
				? json(list)
				: call.url.endsWith('/api/sessions')
					? json({ sessions })
					: call.url.endsWith('/p1/deploy')
						? json({ available: false, publishes: [] })
						: json({ members: [{ user_id: 'u1', name: 'Mario', role: 'owner' }] }),
		);
		expect(await projects.loadProjects(event)).toEqual({
			list,
			members: [{ user_id: 'u1', name: 'Mario', role: 'owner' }],
			chats: [
				{ id: 's1', title: 'Build a menu', updated_at: '2026-11-13T09:00:00Z', project: 'p1', own: true, active: true },
				{ id: 'c2', title: 'Ideas', updated_at: '2026-11-12T09:00:00Z', project: 'p2', own: false, active: false },
			],
			publishing: false,
		});
		// Publishing is asked of the one project that could be published.
		expect(backend.calls.filter((call) => call.url.includes('/deploy')).map((call) => call.url)).toEqual(['http://api.test/api/projects/p1/deploy']);
	});

	it('has no list when the backend can’t give one, and no members before a workspace is chosen', async () => {
		expect(await projects.loadProjects(eventWith(() => new Response('', { status: 500 })).event)).toEqual({ list: null, status: 500, members: [], chats: [], publishing: null });
		const { backend, event } = eventWith(() => json({ org_id: null, projects: [] }));
		expect(await projects.loadProjects(event)).toEqual({ list: { org_id: null, projects: [] }, members: [], chats: [], publishing: null });
		expect(backend.calls).toHaveLength(1);
	});

	it('tells Koda in the project’s own chat', async () => {
		const { backend, event } = eventWith(() => json({ user_message: {} }));
		expect(await projects.tellKoda(event, 's1', 'stop')).toEqual({ ok: true });
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/sessions/s1/messages', body: { text: 'stop' } });
		expect(await projects.tellKoda(eventWith(() => new Response('not yours', { status: 403 })).event, 's1', 'x')).toEqual({ ok: false, status: 403, message: 'not yours' });
	});

	it('says what’s in a project: files, tables, who can open it and its deploys', async () => {
		const { event } = eventWith((call) => {
			if (call.url.endsWith('/api/projects/p1')) return json({ files: [{ path: 'a' }, { path: 'b' }], stack: 'sveltekit', org_id: 'w1', org_name: 'Kopi' });
			if (call.url.endsWith('/export')) return json({ name: 'x', files: 2, bytes: 10, database: 'postgres', env: [] });
			if (call.url.endsWith('/members'))
				return json({
					access: 'members',
					me: 'u2',
					my_role: 'admin',
					people: [
						{ user_id: 'u1', name: 'Trian', is_owner: true, project_role: null },
						{ user_id: 'u2', name: 'Mario', is_owner: false, project_role: 'admin' },
						{ user_id: 'u3', name: 'Sari', is_owner: false, project_role: null },
					],
				});
			return json({
				available: true,
				publishes: [
					{ environment: 'production', status: 'failed', version: 3, is_live: false, created_at: '2026-11-13T09:00:00Z', finished_at: '2026-11-13T09:01:00Z' },
					{ environment: 'production', status: 'succeeded', version: 2, is_live: true, created_at: '2026-11-12T09:00:00Z', finished_at: '2026-11-12T09:00:41Z' },
					{ environment: 'staging', status: 'succeeded', version: 1, is_live: true, created_at: '2026-11-11T09:00:00Z', finished_at: '2026-11-11T09:00:10Z' },
				],
			});
		});
		expect(await projects.projectAbout(event, 'p1')).toEqual({
			files: 2,
			stack: 'sveltekit',
			database: 'postgres',
			people: [
				{ name: 'Trian', owner: true, me: false },
				{ name: 'Mario', owner: false, me: true },
			],
			deploys: { count: 1, builtIn: 41, liveNumber: 2, liveAt: '2026-11-12T09:00:41Z' },
		});
	});

	it('says nothing about a project it can’t open, and leaves out the parts it can’t read', async () => {
		expect(await projects.projectAbout(eventWith(() => new Response('', { status: 404 })).event, 'p1')).toBeNull();
		const { event } = eventWith((call) => (call.url.endsWith('/api/projects/p1') ? json({ files: [], stack: '' }) : new Response('', { status: 503 })));
		expect(await projects.projectAbout(event, 'p1')).toEqual({ files: 0, stack: null, database: null, people: null, deploys: null });
		// The database is read from the files, never by opening it.
		const { backend, event: read } = eventWith(() => json({ files: [], stack: '' }));
		await projects.projectAbout(read, 'p1');
		expect(backend.calls.map((call) => call.url.replace('http://api.test', ''))).toEqual(['/api/projects/p1', '/api/projects/p1/export', '/api/projects/p1/members', '/api/projects/p1/deploy']);
	});

	it('lists where a project can move, with each workspace’s people and the person’s role there', async () => {
		const { event } = eventWith((call) => {
			if (call.url.includes('/transfer/orgs')) return json([{ id: 'w2', name: 'Kopi Kita', owner_name: 'Sari', my_role: 'admin', is_personal: false }]);
			if (call.url.endsWith('/api/orgs'))
				return json({
					orgs: [
						{ id: 'w1', name: 'Trian’s space', role: 'owner', members: 3, projects: 2 },
						{ id: 'w2', name: 'Kopi Kita', role: 'admin', members: 5, projects: 1 },
					],
					project_org_id: 'w1',
				});
			return json({ org_id: 'w1', org_name: 'Trian’s space', files: [] });
		});
		expect(await projects.moveChoices(event, 'p1')).toEqual({
			current: { id: 'w1', name: 'Trian’s space', members: 3 },
			targets: [{ id: 'w2', name: 'Kopi Kita', members: 5, role: 'admin', owner: 'Sari' }],
		});
		expect(await projects.moveChoices(eventWith(() => new Response('', { status: 403 })).event, 'p1')).toBeNull();
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
	it('renames a project and makes a copy of one, passing the backend’s reason on', async () => {
		const { backend, event } = eventWith((call) => (call.method === 'PATCH' ? json({ id: 'p1', name: 'kopi-menu' }) : json({ session_id: 's2', project_id: 'p2' }, 201)));
		expect(await projects.renameProject(event, 'p 1', 'kopi-menu')).toEqual({ ok: true });
		expect(await projects.duplicateProject(event, 'p 1')).toEqual({ ok: true, projectId: 'p2' });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body])).toEqual([
			['PATCH', '/api/projects/p%201', { name: 'kopi-menu' }],
			['POST', '/api/projects/p%201/duplicate', {}],
		]);
		const refused = eventWith(() => new Response('only the project’s builders can rename it', { status: 403 }));
		expect(await projects.renameProject(refused.event, 'p1', 'x')).toEqual({ ok: false, status: 403, message: 'only the project’s builders can rename it' });
		expect(await projects.duplicateProject(refused.event, 'p1')).toEqual({ ok: false, status: 403, message: 'only the project’s builders can rename it' });
	});
});
