import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');
const deploy = await import('./deploy/+server');
const domainCheck = await import('./domain/check/+server');
const transferSearch = await import('./transfer/+server');
const summary = await import('./export/summary/+server');
const preview = await import('./preview/[...path]/+server');
const app = await import('./app/+page.server');
const sandbox = await import('./sandbox/[action]/+server');
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
		expect(await actions.invite(invited.event as never)).toEqual({ invited: 'sari@kopi.id' });
		expect(invited.api.calls[0].body).toEqual({ email: 'sari@kopi.id', role: 'view' });
		expect(await actions.invite(posted({ email: 'nope', role: 'view' }, ok).event as never)).toMatchObject({ status: 400, data: { field: 'email' } });
		expect(await actions.role(posted({ user: 'u2', role: 'viewer' }, ok).event as never)).toEqual({ changed: 'u2' });
		expect(await actions.role(posted({ user: 'u2', role: 'boss' }, ok).event as never)).toMatchObject({ status: 400 });
		expect(await actions.remove(posted({ user: 'u2' }, () => new Response("the project's starter can't be removed", { status: 409 })).event as never)).toMatchObject({ status: 409, data: { message: "the project's starter can't be removed" } });
		expect(await actions.access(posted({ access: 'members' }, ok).event as never)).toEqual({ access: 'members' });
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
});
