import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const workspace = await import('./workspace');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('a project’s workspace on the server', () => {
	it('loads the project and its live address from the list', async () => {
		const { event } = eventWith((call) =>
			call.url.endsWith('/api/projects') ? json({ org_id: 'w', projects: [{ id: 'p1', deploy_url: 'https://warung.trian.space', deploys_count: 2 }], base_domain: 'trian.space' }) : json({ id: 'p1', name: 'warung', session_id: 's1' }),
		);
		expect(await workspace.loadProject(event, 'p1')).toEqual({ project: { id: 'p1', name: 'warung', session_id: 's1' }, live: { url: 'https://warung.trian.space', deploys: 2 }, domain: 'trian.space' });
	});

	it('says when the project isn’t there, or isn’t theirs', async () => {
		const { event } = eventWith(() => new Response('project not found', { status: 404 }));
		expect(await workspace.loadProject(event, 'p1')).toEqual({ project: null, status: 404 });
	});

	it('finds a project by its chat', async () => {
		const { backend, event } = eventWith(() => json({ id: 'p1' }));
		expect(await workspace.projectOfChat(event, 's 1')).toBe('p1');
		expect(backend.calls[0].url).toBe('http://api.test/api/projects/by-session/s%201');
		expect(await workspace.projectOfChat(eventWith(() => new Response('', { status: 404 })).event, 's1')).toBeNull();
	});

	it('loads the deploy overview, publishes and rolls back, passing the backend’s reason on', async () => {
		const { backend, event } = eventWith((call) => (call.method === 'GET' ? json({ available: true, publishes: [] }) : json({}, 202)));
		expect(await workspace.loadDeploy(event, 'p1')).toEqual({ available: true, publishes: [] });
		expect(await workspace.publish(event, 'p1')).toEqual({ ok: true });
		expect(await workspace.rollBack(event, 'p1', 'pub 1')).toEqual({ ok: true });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', '')])).toEqual([
			['GET', '/api/projects/p1/deploy'],
			['POST', '/api/projects/p1/deploy/publish'],
			['POST', '/api/projects/p1/deploy/publishes/pub%201/rollback'],
		]);
		const busy = eventWith(() => new Response('a publish is already running', { status: 409 }));
		expect(await workspace.publish(busy.event, 'p1')).toEqual({ ok: false, status: 409, message: 'a publish is already running' });
	});

	it('reads, checks and changes the app’s address, with a refusal’s free names', async () => {
		const { backend, event } = eventWith((call) =>
			call.method === 'GET' && call.url.includes('/check') ? json({ available: false, message: 'taken', suggestions: ['kopi-1'] }) : call.method === 'GET' ? json({ hostname: null, can_edit: true }) : json({ url: 'https://kopi.trian.space' }),
		);
		expect(await workspace.loadDomain(event, 'p1')).toEqual({ hostname: null, can_edit: true });
		expect(await workspace.checkDomain(event, 'p1', 'kopi bu')).toEqual({ available: false, message: 'taken', suggestions: ['kopi-1'] });
		expect(await workspace.changeDomain(event, 'p1', 'kopi')).toEqual({ ok: true, url: 'https://kopi.trian.space' });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body ?? null])).toEqual([
			['GET', '/api/projects/p1/domain', null],
			['GET', '/api/projects/p1/domain/check?subdomain=kopi%20bu', null],
			['PUT', '/api/projects/p1/domain', { subdomain: 'kopi' }],
		]);
		const refused = eventWith(() => json({ available: false, message: 'taken a moment ago', suggestions: ['kopi-2'] }, 409));
		expect(await workspace.changeDomain(refused.event, 'p1', 'kopi')).toEqual({ ok: false, status: 409, answer: { available: false, message: 'taken a moment ago', suggestions: ['kopi-2'] }, message: 'taken a moment ago' });
		const first = eventWith(() => new Response('publish the app first: it gets an address on its first publish', { status: 409 }));
		expect(await workspace.changeDomain(first.event, 'p1', 'kopi')).toMatchObject({ ok: false, status: 409, message: 'publish the app first: it gets an address on its first publish' });
	});

	it('gathers the running app’s numbers, what it uses and its last lines, each on its own', async () => {
		const { backend, event } = eventWith((call) =>
			call.url.includes('/metrics') ? json({ range: '1h', buckets: [] }) : call.url.includes('/usage') ? new Response('', { status: 502 }) : json({ lines: ['GET /menu 200'], note: 'tail' }),
		);
		expect(await workspace.loadMonitor(event, 'p1', '1h')).toEqual({ metrics: { range: '1h', buckets: [] }, usage: null, logs: ['GET /menu 200'], note: 'tail' });
		expect(backend.calls.map((call) => call.url.replace('http://api.test', '')).sort()).toEqual(['/api/projects/p1/deploy/logs?tail=40', '/api/projects/p1/deploy/metrics?range=1h', '/api/projects/p1/deploy/usage']);
	});

	it('lists the tables and reads the one asked for, with a statement it builds itself', async () => {
		const { backend, event } = eventWith((call) =>
			call.url.includes('/tables') ? json([{ schema: 'public', name: 'menu', rows_estimate: 10 }, { schema: 'public', name: 'orders', rows_estimate: 7 }]) : json({ columns: ['id'], rows: [[7]], truncated: false, rows_affected: null, took_ms: 3 }),
		);
		expect(await workspace.readDatabase(event, 'p1', 'production', 'public.orders')).toMatchObject({ table: { name: 'orders' }, rows: { columns: ['id'], rows: [[7]] } });
		expect(backend.calls[1]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/database/query', body: { sql: 'SELECT * FROM "public"."orders" LIMIT 50', source: 'production' } });
		const picked = eventWith((call) => (call.url.includes('/tables') ? json([{ schema: 'public', name: 'menu', rows_estimate: 10 }]) : json({ columns: [], rows: [], truncated: false, rows_affected: null, took_ms: 1 })));
		expect(await workspace.readDatabase(picked.event, 'p1', 'development', 'public."; DROP TABLE x; --')).toMatchObject({ table: { name: 'menu' } });
		expect(picked.backend.calls[1].body).toMatchObject({ sql: 'SELECT * FROM "public"."menu" LIMIT 50' });
	});

	it('passes on why the database can’t be read', async () => {
		const { event } = eventWith(() => new Response('This project does not use a database.', { status: 404 }));
		expect(await workspace.readDatabase(event, 'p1', 'development', null)).toEqual({ error: 'This project does not use a database.', status: 404 });
	});

	it('finds workspaces to move to, transfers, and takes the app down', async () => {
		const { backend, event } = eventWith((call) => (call.method === 'GET' ? json([{ id: 'w2', name: 'Kopi Kita', is_personal: false, owner_name: 'Siti', my_role: 'admin' }]) : new Response(null, { status: 204 })));
		expect(await workspace.transferTargets(event, 'p1', 'kopi k')).toEqual([{ id: 'w2', name: 'Kopi Kita', is_personal: false, owner_name: 'Siti', my_role: 'admin' }]);
		expect(await workspace.transfer(event, 'p1', 'w2', 'warung')).toEqual({ ok: true });
		expect(await workspace.takeDown(event, 'p1')).toEqual({ ok: true });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body ?? null])).toEqual([
			['GET', '/api/projects/p1/transfer/orgs?q=kopi%20k', null],
			['POST', '/api/projects/p1/transfer', { target_org_id: 'w2', confirm_name: 'warung' }],
			['DELETE', '/api/projects/p1/deploy/environments/production', null],
		]);
		const wrong = eventWith(() => new Response("type the project's name to confirm the transfer", { status: 400 }));
		expect(await workspace.transfer(wrong.event, 'p1', 'w2', 'x')).toEqual({ ok: false, status: 400, message: "type the project's name to confirm the transfer" });
	});

	it('reads the people and the environment, and changes each on its own path', async () => {
		const { backend, event } = eventWith((call) => (call.url.endsWith('/members') ? json({ access: 'org', people: [] }) : call.url.endsWith('/secrets') ? json({ secrets: [], can_edit: true }) : new Response(null, { status: 204 })));
		expect(await workspace.loadPeople(event, 'p1')).toEqual({ access: 'org', people: [] });
		expect(await workspace.loadSecrets(event, 'p1')).toEqual({ secrets: [], can_edit: true });
		await workspace.invitePerson(event, 'p1', 'sari@kopi.id', 'build');
		await workspace.setPersonRole(event, 'p1', 'u 2', 'viewer');
		await workspace.removePerson(event, 'p1', 'u2');
		await workspace.setAccess(event, 'p1', 'members');
		await workspace.setSecret(event, 'p1', 'ADMIN_PIN', '1234');
		await workspace.removeSecret(event, 'p1', 'ADMIN_PIN');
		expect(backend.calls.slice(2).map((call) => [call.method, call.url.replace('http://api.test', ''), call.body ?? null])).toEqual([
			['POST', '/api/projects/p1/invites', { email: 'sari@kopi.id', role: 'build' }],
			['PUT', '/api/projects/p1/members/u%202', { role: 'viewer' }],
			['DELETE', '/api/projects/p1/members/u2', null],
			['PUT', '/api/projects/p1/access', { access: 'members' }],
			['PUT', '/api/projects/p1/secrets/ADMIN_PIN', { value: '1234' }],
			['DELETE', '/api/projects/p1/secrets/ADMIN_PIN', null],
		]);
	});
});
