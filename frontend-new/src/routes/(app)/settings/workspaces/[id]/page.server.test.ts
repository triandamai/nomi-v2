import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const detail = { id: 'w1', name: 'Kopi Kita', my_role: 'owner', me: 'me', members: [], invites: [] };
const at = (answer: Parameters<typeof fakeBackend>[0], form?: Record<string, string>) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ url: 'http://localhost/settings/workspaces/w1', params: { id: 'w1' }, form, fetch: api.fetch, locals: { accessToken: 'token' } }) };
};

describe('Settings › Workspace', () => {
	it('names itself after the workspace', async () => {
		const { event } = at((call) => (call.url.endsWith('/api/orgs') ? json({ orgs: [], project_org_id: null }) : json(detail)));
		expect(await load(event as never)).toMatchObject({ title: 'Kopi Kita', detail: { my_role: 'owner' } });
	});

	it('is not found when the person isn’t in it', async () => {
		const { event } = at(() => new Response('not a member', { status: 403 }));
		expect(await thrown(() => load(event as never))).toMatchObject({ status: 404 });
	});

	it('invites every address it was given, and says which didn’t go', async () => {
		const { api, event } = at(
			(call) => ((call.body as { email: string }).email === 'taken@kopi.id' ? new Response("they're already a member", { status: 409 }) : json({ code: 'c' }, 201)),
			{ emails: 'siti@kopi.id, taken@kopi.id, nope', role: 'admin' },
		);
		expect(await actions.invite(event as never)).toMatchObject({ status: 400, data: { field: 'emails', message: 'Check these addresses: nope' } });
		expect(api.calls).toEqual([]);
		const second = at(
			(call) => ((call.body as { email: string }).email === 'taken@kopi.id' ? new Response("they're already a member", { status: 409 }) : json({ code: 'c' }, 201)),
			{ emails: 'siti@kopi.id, taken@kopi.id', role: 'admin' },
		);
		expect(await actions.invite(second.event as never)).toEqual({ sent: 1, failed: [{ email: 'taken@kopi.id', message: "they're already a member" }] });
		expect(second.api.calls.map((call) => call.body)).toEqual([{ email: 'siti@kopi.id', role: 'admin' }, { email: 'taken@kopi.id', role: 'admin' }]);
	});

	it('renames, and asks for a name', async () => {
		expect(await actions.rename(at(() => new Response(null, { status: 204 }), { name: 'Kopi' }).event as never)).toEqual({ renamed: 'Kopi' });
		expect(await actions.rename(at(() => new Response(null, { status: 204 }), { name: ' ' }).event as never)).toMatchObject({ status: 400 });
	});

	it('changes a role, removes someone, and passes on the backend’s refusal', async () => {
		expect(await actions.role(at(() => new Response(null, { status: 204 }), { user: 'u2', role: 'admin' }).event as never)).toEqual({ changed: 'u2' });
		expect(await actions.role(at(() => new Response(null, { status: 204 }), { user: 'u2', role: 'boss' }).event as never)).toMatchObject({ status: 400 });
		expect(await actions.remove(at(() => new Response('only owners can remove an owner', { status: 403 }), { user: 'u2' }).event as never)).toMatchObject({ status: 403, data: { message: 'only owners can remove an owner' } });
		expect(await actions.revoke(at(() => new Response(null, { status: 204 }), { code: 'c1' }).event as never)).toEqual({ revoked: 'c1' });
	});

	it('leaves, then goes back to the workspaces', async () => {
		const { api, event } = at((call) => (call.method !== 'GET' ? new Response(null, { status: 204 }) : call.url.endsWith('/api/orgs') ? json({ orgs: [], project_org_id: null }) : json(detail)));
		expect(await thrown(() => actions.leave(event as never))).toMatchObject({ status: 303, location: '/settings/workspaces?left=Kopi%20Kita' });
		expect(api.calls.at(-1)).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/orgs/w1/members/me' });
	});
});
