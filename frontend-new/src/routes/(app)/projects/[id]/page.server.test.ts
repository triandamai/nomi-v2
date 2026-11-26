import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const deploy = await import('./deploy/+server');
const domainCheck = await import('./domain/check/+server');
const transferSearch = await import('./transfer/+server');
const summary = await import('./export/summary/+server');
const preview = await import('./preview/[...path]/+server');
const { sealPreview } = await import('$lib/server/previewGrant');
const app = await import('./app/+page.server');
const sandbox = await import('./sandbox/[action]/+server');
const presence = await import('./presence/+server');
const dump = await import('./database/dump/+server');
const rows = await import('./database/rows/+server');
const queue = await import('./queue/+server');
const queueAsk = await import('./queue/[job]/+server');
const requests = await import('./requests/+server');
const bySession = await import('../session/[sessionId]/+page.server');

const at = (answer: Parameters<typeof fakeBackend>[0], params: Record<string, string>) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ url: 'http://localhost/projects/p1', params, fetch: api.fetch, locals: { accessToken: 'token' } }) };
};

describe('a project’s workspace', () => {
	it('is not found when the project isn’t there', async () => {
		expect(await thrown(() => load(at(() => new Response('', { status: 404 }), { id: 'p1' }).event as never))).toMatchObject({ status: 404 });
	});

	it('serves the project’s files to a grant in the address, sandboxed, and nothing without one', async () => {
		const grant = sealPreview('p1', 'token-1');
		const { api, event } = at(() => new Response('<h1>Warung</h1>', { headers: { 'content-type': 'text/html' } }), { id: 'p1', path: `~${grant}/` });
		const response = await preview.GET({ ...event, locals: {} } as never);
		expect([response.status, response.headers.get('content-type'), await response.text()]).toEqual([200, 'text/html', '<h1>Warung</h1>']);
		expect(api.calls[0]).toMatchObject({ url: 'http://api.test/api/projects/p1/preview', headers: { authorization: 'Bearer token-1' } });
		// The project's files are code Koda wrote: they run in a sandbox of their own origin, never as this app.
		expect(response.headers.get('content-security-policy')).toBe('sandbox allow-scripts');
		expect(response.headers.get('x-content-type-options')).toBe('nosniff');
		expect(response.headers.get('referrer-policy')).toBe('no-referrer');
		const asset = at(() => new Response('body{}', { headers: { 'content-type': 'text/css' } }), { id: 'p1', path: `~${grant}/css/app.css` });
		await preview.GET(asset.event as never);
		expect(asset.api.calls[0].url).toBe('http://api.test/api/projects/p1/preview/css/app.css');
		for (const path of ['', 'index.html', `~${grant}`, `~${grant}/../../users`, `~${grant}/./a.css`, `~${sealPreview('p2', 'token-1')}/`, '~made-up/']) {
			const refused = at(() => new Response('<h1>Warung</h1>'), { id: 'p1', path });
			expect((await preview.GET(refused.event as never)).status, path).toBe(403);
			expect(refused.api.calls).toEqual([]);
		}
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

	const posted = (body: Record<string, string>, answer: Parameters<typeof fakeBackend>[0]) => {
		const { api, event } = at(answer, { id: 'p1' });
		return { api, event: { ...event, request: new Request('http://localhost/projects/p1', { method: 'POST', body: new URLSearchParams(body) }) } };
	};

	it('transfers to a workspace, then leaves for Projects', async () => {
		const done = posted({ to: 'w2', name: 'warung' }, () => new Response(null, { status: 204 }));
		expect(await thrown(() => actions.transfer(done.event as never))).toMatchObject({ status: 303, location: '/projects?moved=warung' });
		expect(done.api.calls[0].body).toEqual({ target_org_id: 'w2', confirm_name: 'warung' });
		expect(await actions.transfer(posted({ name: 'warung' }, () => new Response(null, { status: 204 })).event as never)).toMatchObject({ status: 400 });
	});

	it('takes the app down', async () => {
		const down = posted({}, () => new Response(null, { status: 204 }));
		expect(await actions.takedown(down.event as never)).toEqual({ takenDown: true });
		expect(down.api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/projects/p1/deploy/environments/production' });
	});

	it('deletes only once the project’s name is typed', async () => {
		const project = (call: { method: string; url: string }) =>
			call.method === 'DELETE' ? new Response(null, { status: 204 }) : call.url.endsWith('/api/projects') ? json({ org_id: 'w', projects: [] }) : json({ id: 'p1', name: 'warung', session_id: 's1' });
		expect(await actions.delete(posted({ name: 'warun' }, project).event as never)).toMatchObject({ status: 400, data: { field: 'name' } });
		const gone = posted({ name: 'warung' }, project);
		expect(await thrown(() => actions.delete(gone.event as never))).toMatchObject({ status: 303, location: '/projects?deleted=warung' });
		expect(gone.api.calls.at(-1)).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/projects/p1' });
	});

	it('finds workspaces to move to', async () => {
		const { api, event } = at(() => json([{ id: 'w2', name: 'Kopi Kita' }]), { id: 'p1' });
		const response = await transferSearch.GET({ ...event, url: new URL('http://localhost/projects/p1/transfer?q=kopi') } as never);
		expect(await response.json()).toEqual([{ id: 'w2', name: 'Kopi Kita' }]);
		expect(api.calls[0].url).toBe('http://api.test/api/projects/p1/transfer/orgs?q=kopi');
	});

	it('invites, changes roles, removes and limits access, refusing what’s malformed', async () => {
		const ok = () => new Response(null, { status: 204 });
		const invited = posted({ email: ' Sari@Kopi.id ', role: 'view' }, ok);
		expect(await actions.invite(invited.event as never)).toMatchObject({ invited: 'sari@kopi.id' });
		expect(invited.api.calls[0].body).toEqual({ email: 'sari@kopi.id', role: 'view' });
		expect(await actions.invite(posted({ email: 'nope', role: 'view' }, ok).event as never)).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(await actions.role(posted({ user: 'u2', role: 'viewer' }, ok).event as never)).toEqual({ changed: 'u2' });
		expect(await actions.role(posted({ user: 'u2', role: 'boss' }, ok).event as never)).toMatchObject({ status: 400 });
		expect(await actions.remove(posted({ user: 'u2' }, () => new Response("the project's starter can't be removed", { status: 409 })).event as never)).toMatchObject({ status: 409, data: { message: "the project's starter can't be removed" } });
		expect(await actions.access(posted({ access: 'members' }, ok).event as never)).toEqual({ access: 'members' });
	});

	it('gives back an invite’s link and when it runs out, or that the person was added at once (G10)', async () => {
		const sent = posted({ email: 'sari@kopi.id', role: 'prompt' }, () => json({ added: false, email: 'sari@kopi.id', role: 'prompt', path: '/invite/abc123', expires_at: '2026-11-25T08:00:00Z' }, 201));
		expect(await actions.invite(sent.event as never)).toEqual({ invited: 'sari@kopi.id', added: false, path: '/invite/abc123', expires_at: '2026-11-25T08:00:00Z' });
		const added = posted({ email: 'bima@kopi.id', role: 'view' }, () => json({ added: true, email: 'bima@kopi.id', role: 'view', path: null, expires_at: null }));
		expect(await actions.invite(added.event as never)).toEqual({ invited: 'bima@kopi.id', added: true, path: null, expires_at: null });
	});

	it('sets a variable with a proper name, and says the backend’s reason under the value', async () => {
		const ok = () => new Response(null, { status: 204 });
		const set = posted({ name: 'admin_pin', value: '1234' }, ok);
		expect(await actions.secret(set.event as never)).toEqual({ saved: 'ADMIN_PIN' });
		expect(set.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/projects/p1/secrets/ADMIN_PIN', body: { value: '1234' } });
		expect(await actions.secret(posted({ name: '1bad', value: 'x' }, ok).event as never)).toMatchObject({ status: 400, data: { field: 'name' } });
		expect(await actions.secret(posted({ name: 'X', value: 'v' }, () => new Response("publishing isn't set up on this Nomi", { status: 503 })).event as never)).toMatchObject({ status: 503, data: { field: 'value', message: "publishing isn't set up on this Nomi" } });
		expect(await actions.unsecret(posted({ name: 'X' }, ok).event as never)).toEqual({ removed: 'X' });
	});

	it('pushes to a repository named owner/name, and shows the app or not', async () => {
		const ok = () => new Response(null, { status: 204 });
		const pushed = posted({ repo: 'mario/warung-kopi', create: 'true', private: 'true' }, ok);
		expect(await actions.github(pushed.event as never)).toEqual({ pushed: 'mario/warung-kopi' });
		expect(pushed.api.calls[0].body).toEqual({ repo: 'mario/warung-kopi', create: true, private: true });
		expect(await actions.github(posted({ repo: 'not a repo' }, ok).event as never)).toMatchObject({ status: 400, data: { field: 'repo' } });
		expect(await actions.github(posted({ repo: 'mario/x' }, () => new Response('connect GitHub first', { status: 409 })).event as never)).toMatchObject({ status: 409, data: { message: 'connect GitHub first' } });
		const shown = posted({ showcase: 'true' }, ok);
		expect(await actions.showcase(shown.event as never)).toEqual({ showcase: true });
		expect(shown.api.calls[0].body).toEqual({ showcase: true, blurb: null });
	});

	it('says what an export holds', async () => {
		const response = await summary.GET(at(() => json({ name: 'warung', files: 9, bytes: 1, database: 'none', env: [] }), { id: 'p1' }).event as never);
		expect(await response.json()).toMatchObject({ files: 9 });
	});
	it('passes the sandbox’s status, logs and heartbeat through, starts and stops it, and nothing else', async () => {
		const answer = (call: { url: string }) => (call.url.includes('/logs') ? json({ text: 'ready\n', next_offset: 6 }) : json({ runtime: 'sandbox', available: true, phase: 'running', preview_url: 'https://boat.example/p/t', error: null }));
		const status = at(answer, { id: 'p1', action: 'status' });
		expect(await (await sandbox.GET(status.event as never)).json()).toMatchObject({ phase: 'running' });
		const logs = at(answer, { id: 'p1', action: 'logs' });
		(logs.event as { url: URL }).url = new URL('http://localhost/projects/p1/sandbox/logs?offset=12');
		expect(await (await sandbox.GET(logs.event as never)).json()).toEqual({ text: 'ready\n', next_offset: 6 });
		expect(logs.api.calls[0].url).toBe('http://api.test/api/projects/p1/sandbox/logs?offset=12');
		const beat = at(answer, { id: 'p1', action: 'beat' });
		expect((await sandbox.GET(beat.event as never)).status).toBe(204);
		expect(beat.api.calls[0].url).toBe('http://api.test/api/projects/p1/runtime?runtime=sandbox');
		const start = at(() => json({}), { id: 'p1', action: 'start' });
		expect((await sandbox.POST(start.event as never)).status).toBe(204);
		expect(start.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/sandbox/start' });
		const refused = at(() => new Response('the sandbox isn’t available right now', { status: 409 }), { id: 'p1', action: 'stop' });
		const answered = await sandbox.POST(refused.event as never);
		expect([answered.status, await answered.text()]).toEqual([409, 'the sandbox isn’t available right now']);
		const other = at(answer, { id: 'p1', action: 'files' });
		expect((await sandbox.GET(other.event as never)).status).toBe(404);
		expect((await sandbox.POST(at(answer, { id: 'p1', action: 'status' }).event as never)).status).toBe(404);
		expect(other.api.calls).toEqual([]);
		const gone = at(() => new Response('', { status: 404 }), { id: 'p1', action: 'status' });
		expect((await sandbox.GET(gone.event as never)).status).toBe(503);
	});
	it('passes on who else is here, for the page to show', async () => {
		const here = at(() => json({ me: 'u-me', people: [] }), { id: 'p1' });
		expect(await (await presence.GET(here.event as never)).json()).toEqual({ me: 'u-me', people: [] });
		expect((await presence.GET(at(() => new Response('', { status: 404 }), { id: 'p1' }).event as never)).status).toBe(503);
	});
	it('passes on the asks in line, and lets one go first with the line as it is now or why not', async () => {
		const line = { asks: [{ id: 'j1', text: 'Make it green', who: 'Mario', mine: true, state: 'running', created_at: '2026-11-14T02:14:00Z' }] };
		const answer = (call: { url: string }) => (call.url.includes('/queue') ? json(line) : json({ id: 'p1', session_id: 's1' }));
		expect(await (await queue.GET(at(answer, { id: 'p1' }).event as never)).json()).toEqual(line);
		expect((await queue.GET(at(() => new Response('', { status: 404 }), { id: 'p1' }).event as never)).status).toBe(503);
		const moved = at(answer, { id: 'p1', job: 'j1' });
		expect(await (await queueAsk.POST(moved.event as never)).json()).toEqual(line);
		expect(moved.api.calls[1]).toMatchObject({ method: 'POST', url: 'http://api.test/api/sessions/s1/queue/j1/first' });
		const late = at((call) => (call.url.endsWith('/first') ? new Response('that ask isn’t waiting any more', { status: 409 }) : json({ id: 'p1', session_id: 's1' })), { id: 'p1', job: 'j1' });
		const refused = await queueAsk.POST(late.event as never);
		expect([refused.status, await refused.json()]).toEqual([409, { message: 'that ask isn’t waiting any more' }]);
		const removed = at(answer, { id: 'p1', job: 'j2' });
		expect(await (await queueAsk.DELETE(removed.event as never)).json()).toEqual(line);
		expect(removed.api.calls[1]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/sessions/s1/queue/j2' });
		const busy = at((call) => (call.method === 'DELETE' ? new Response('that ask isn’t waiting any more', { status: 409 }) : json({ id: 'p1', session_id: 's1' })), { id: 'p1', job: 'j1' });
		const cantRemove = await queueAsk.DELETE(busy.event as never);
		expect([cantRemove.status, await cantRemove.json()]).toEqual([409, { message: 'that ask isn’t waiting any more' }]);
	});
	it('passes on the requests to publish', async () => {
		const rows = { requests: [{ id: 'r1', status: 'pending' }] };
		expect(await (await requests.GET(at(() => json(rows), { id: 'p1' }).event as never)).json()).toEqual(rows);
		expect((await requests.GET(at(() => new Response('', { status: 404 }), { id: 'p1' }).event as never)).status).toBe(503);
	});
	it('asks to publish, and answers a request, saying why not in the backend’s words', async () => {
		const asked = at(() => json({ id: 'r1' }, 201), { id: 'p1' });
		expect(await actions.ask(asked.event as never)).toEqual({ asked: true });
		expect(asked.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/deploy/requests', body: {} });
		expect(await actions.ask(at(() => new Response('you’ve already asked; it’s waiting for an admin', { status: 409 }), { id: 'p1' }).event as never)).toMatchObject({
			status: 409,
			data: { message: 'you’ve already asked; it’s waiting for an admin' },
		});
		const form = (fields: Record<string, string>, answer: Parameters<typeof at>[0]) => {
			const one = at(answer, { id: 'p1' });
			return { one, event: { ...one.event, request: new Request('http://localhost/projects/p1', { method: 'POST', body: new URLSearchParams(fields) }) } };
		};
		const approve = form({ request: 'r1', action: 'approve' }, () => json({ id: 'r1', status: 'approved' }));
		expect(await actions.answer(approve.event as never)).toEqual({ answered: 'approve' });
		expect(approve.one.api.calls[0]).toMatchObject({ url: 'http://api.test/api/projects/p1/deploy/requests/r1', body: { action: 'approve' } });
		const off = form({ request: 'r1', action: 'approve' }, () => new Response('publishing isn’t set up on this Nomi', { status: 503 }));
		expect(await actions.answer(off.event as never)).toMatchObject({ status: 503, data: { message: 'publishing isn’t set up on this Nomi' } });
		const odd = form({ request: 'r1', action: 'merge' }, () => json({}));
		expect(await actions.answer(odd.event as never)).toMatchObject({ status: 400 });
		expect(odd.one.api.calls).toEqual([]);
	});
	it('adds, checks, makes primary and removes an own domain, saying why not', async () => {
		const form = (body: Record<string, string>, answer: Parameters<typeof fakeBackend>[0]) => {
			const { api, event } = at(answer, { id: 'p1' });
			return { api, event: { ...event, request: new Request('http://localhost/projects/p1', { method: 'POST', body: new URLSearchParams(body) }) } };
		};
		const added = form({ hostname: ' WarungKopi.ID ' }, () => json({}, 201));
		expect(await actions.domainAdd(added.event as never)).toEqual({ domain: 'warungkopi.id' });
		expect(added.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/projects/p1/domains', body: { hostname: 'warungkopi.id' } });
		expect(await actions.domainAdd(form({ hostname: ' ' }, () => json({})).event as never)).toMatchObject({ status: 400 });
		expect(await actions.domainAdd(form({ hostname: 'x.id' }, () => new Response('your plan doesn’t include your own domains', { status: 402 })).event as never)).toMatchObject({ status: 402, data: { message: 'your plan doesn’t include your own domains' } });
		for (const [action, method, path] of [['domainCheck', 'POST', '/check'], ['domainPrimary', 'PUT', '/primary'], ['domainRemove', 'DELETE', '']] as const) {
			const done = form({ hostname: 'warungkopi.id' }, () => (method === 'DELETE' ? new Response(null, { status: 204 }) : json({})));
			expect(await actions[action](done.event as never)).toEqual({ domain: 'warungkopi.id' });
			expect(done.api.calls[0]).toMatchObject({ method, url: `http://api.test/api/projects/p1/domains/warungkopi.id${path}` });
		}
	});
	it('downloads a dump as the backend names it, from the database asked for', async () => {
		const { api, event } = at(() => new Response('-- dump', { headers: { 'content-type': 'application/sql', 'content-disposition': 'attachment; filename="warung-production-2026-11-15.sql"' } }), { id: 'p1' });
		const response = await dump.GET({ ...event, url: new URL('http://localhost/projects/p1/database/dump?source=production') } as never);
		expect([response.status, response.headers.get('content-disposition'), await response.text()]).toEqual([200, 'attachment; filename="warung-production-2026-11-15.sql"', '-- dump']);
		expect(api.calls[0].url).toBe('http://api.test/api/projects/p1/database/dump?source=production');
		const other = at(() => new Response('-- dump'), { id: 'p1' });
		await dump.GET({ ...other.event, url: new URL('http://localhost/projects/p1/database/dump?source=everything') } as never);
		expect(other.api.calls[0].url).toBe('http://api.test/api/projects/p1/database/dump?source=development');
	});

	it('edits a row from the grid, and refuses a body that isn’t one', async () => {
		const send = (body: unknown, answer: Parameters<typeof fakeBackend>[0] = () => json({ row: { id: 7 } })) => {
			const { api, event } = at(answer, { id: 'p1' });
			const request = new Request('http://localhost/projects/p1/database/rows', { method: 'POST', body: JSON.stringify(body) });
			return { api, done: rows.POST({ ...event, request } as never) };
		};
		const ok = send({ source: 'development', schema: 'public', table: 'orders', key: { id: 7 }, changes: { status: 'picked up' } });
		const answered = await ok.done;
		expect([answered.status, await answered.json()]).toEqual([200, { row: { id: 7 } }]);
		expect(ok.api.calls[0]).toMatchObject({ method: 'PATCH', url: 'http://api.test/api/projects/p1/database/tables/public/orders/rows' });
		for (const bad of [{}, { source: 'development', schema: 'public', table: 'orders', key: [], changes: {} }, { source: 'development', schema: 1, table: 'orders', key: {}, changes: {} }, 'nope']) {
			const refused = send(bad);
			expect((await refused.done).status).toBe(400);
			expect(refused.api.calls).toEqual([]);
		}
		const reason = await send({ source: 'production', schema: 'public', table: 'orders', key: { id: 7 }, changes: { total: 'abc' } }, () => new Response('invalid input', { status: 422 })).done;
		expect([reason.status, await reason.json()]).toEqual([422, { error: 'invalid input' }]);
	});
});
