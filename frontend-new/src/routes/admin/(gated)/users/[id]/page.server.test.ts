import { describe, expect, it, vi } from 'vitest';
import type { Access } from '$lib/admin/access';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const detail = { id: 'u1', email: 'ana@kopikita.id', is_platform_admin: false, display_name: 'Ana', username: 'ana', permissions: [], memberships: [] };

function event(options: { access: Access; form?: Record<string, string>; answer?: (call: BackendCall) => Response }) {
	const api = fakeBackend(options.answer ?? ((call) => (call.url.endsWith('/api/admin/users/u1') ? json(detail) : json([]))));
	const base = fakeEvent({ url: 'http://localhost/admin/users/u1', params: { id: 'u1' }, form: options.form, fetch: api.fetch, locals: { accessToken: 't' } });
	return { api, event: { ...base, parent: async () => ({ access: options.access }) } as never };
}

const urls = (api: { calls: BackendCall[] }) => api.calls.map((call) => `${call.method} ${call.url.replace('http://api.test', '')}`);

describe('a person in the console', () => {
	it('loads the person, and only what the viewer may see beside them', async () => {
		const viewer = event({ access: { user: 'view' } });
		const result = await load(viewer.event);
		expect(urls(viewer.api)).toEqual(['GET /api/admin/users/u1']);
		expect(result).toMatchObject({ title: 'ana@kopikita.id', person: { ok: true, data: detail }, orgs: null, subscription: null, plans: null });

		const editor = event({ access: { user: 'write', plans: 'write' } });
		await load(editor.event);
		expect(urls(editor.api).sort()).toEqual(['GET /api/admin/orgs', 'GET /api/admin/plans', 'GET /api/admin/users/u1', 'GET /api/admin/users/u1/subscription']);
	});

	it('is a 404 for someone who isn’t there', async () => {
		const { event: missing } = event({ access: { user: 'view' }, answer: () => new Response('user not found', { status: 404 }) });
		await expect(load(missing)).rejects.toMatchObject({ status: 404 });
	});

	it('saves the profile, an empty username clearing it', async () => {
		const { api, event: saving } = event({ access: { user: 'write' }, form: { display_name: ' Ana Lestari ', username: '' }, answer: () => json(detail) });
		expect(await actions.profile(saving)).toMatchObject({ action: 'profile', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/users/u1/profile', body: { display_name: 'Ana Lestari', username: '' } });
	});

	it('sets the admin role as a whole, from the editor', async () => {
		const { api, event: saving } = event({ access: { user: 'manage' }, form: { access: JSON.stringify({ plans: 'write', support: 'view', tools: null }) }, answer: () => json([]) });
		expect(await actions.access(saving)).toMatchObject({ action: 'access', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/users/u1/admin-access', body: { access: [{ resource: 'plans', level: 'write' }, { resource: 'support', level: 'view' }] } });
	});

	it('refuses an admin role it can’t read, before calling the backend', async () => {
		const { api, event: saving } = event({ access: { user: 'manage' }, form: { access: 'nope' } });
		expect(await actions.access(saving)).toMatchObject({ status: 400, data: { action: 'access', ok: false } });
		expect(api.calls).toEqual([]);
	});

	it('adds them to a workspace in a role, and takes them out of one', async () => {
		const adding = event({ access: { user: 'write' }, form: { org_id: 'o1', role: 'admin' }, answer: () => json([]) });
		expect(await actions.addMembership(adding.event)).toMatchObject({ action: 'addMembership', ok: true });
		expect(adding.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/users/u1/memberships', body: { org_id: 'o1', role: 'admin' } });

		const wrong = event({ access: { user: 'write' }, form: { org_id: 'o1', role: 'boss' } });
		expect(await actions.addMembership(wrong.event)).toMatchObject({ status: 400 });
		expect(wrong.api.calls).toEqual([]);

		const removing = event({ access: { user: 'write' }, form: { org_id: 'o1' }, answer: () => json([]) });
		expect(await actions.removeMembership(removing.event)).toMatchObject({ action: 'removeMembership', ok: true });
		expect(removing.api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/admin/users/u1/memberships/o1' });
	});

	it('sets the plan and an allowance of its own until a day, or the plan’s', async () => {
		const custom = event({ access: { plans: 'write' }, form: { plan_id: 'p2', quota_override: '2 000 000', override_until: '2026-12-31', note: 'Pilot' }, answer: () => json({}) });
		expect(await actions.subscription(custom.event)).toMatchObject({ action: 'subscription', ok: true });
		expect(custom.api.calls[0]).toMatchObject({
			method: 'PUT',
			url: 'http://api.test/api/admin/users/u1/subscription',
			body: { plan_id: 'p2', quota_override: 2_000_000, override_until: '2026-12-31T23:59:59.000Z', note: 'Pilot' },
		});

		const plain = event({ access: { plans: 'write' }, form: { plan_id: 'p1', quota_override: '', override_until: '', note: '' }, answer: () => json({}) });
		await actions.subscription(plain.event);
		expect(plain.api.calls[0].body).toEqual({ plan_id: 'p1', quota_override: null, override_until: null, note: null });

		for (const form of [
			{ plan_id: '', quota_override: '', override_until: '', note: '' },
			{ plan_id: 'p1', quota_override: 'lots', override_until: '', note: '' },
			{ plan_id: 'p1', quota_override: '5', override_until: '31/12/2026', note: '' },
		]) {
			const bad = event({ access: { plans: 'write' }, form });
			expect(await actions.subscription(bad.event)).toMatchObject({ status: 400, data: { action: 'subscription' } });
			expect(bad.api.calls).toEqual([]);
		}
	});

	it('revokes a workspace grant', async () => {
		const { api, event: revoking } = event({ access: { user: 'manage' }, form: { permission_id: 'g1' }, answer: () => json([]) });
		expect(await actions.revoke(revoking)).toMatchObject({ action: 'revoke', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/admin/users/u1/permissions/g1' });
	});

	it('hands back the backend’s reason when it refuses a change', async () => {
		const { event: refused } = event({ access: { user: 'manage' }, form: { access: '{}' }, answer: () => json({ error: 'cannot grant admin:plans:manage beyond your own' }, 403) });
		expect(await actions.access(refused)).toMatchObject({ status: 403, data: { ok: false, problem: { message: 'cannot grant admin:plans:manage beyond your own' } } });
	});
});
