import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json, settle } from '../../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const orgData = {
	id: 'org1',
	name: 'Acme',
	my_role: 'owner',
	me: 'u1',
	members: [{ user_id: 'u1', name: 'Ana', email: 'ana@example.com', role: 'owner', joined_at: '2026-01-01' }],
	invites: [{ code: 'c1', email: 'bob@example.com', role: 'member', created_at: '2026-01-01', expires_at: '2026-02-01' }],
	is_personal: false,
};

describe('organization [orgId] page.server', () => {
	it('load fetches organization details or throws error', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const mockFetch = vi.fn(async () => json(orgData));
		vi.stubGlobal('fetch', mockFetch);

		const data = await load(event({ cookies, params: { orgId: 'org1' } }));
		expect(data).toMatchObject({ org: { name: 'Acme' } });

		// 404 case
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 404)));
		const err = await settle(() => load(event({ cookies, params: { orgId: 'org1' } })));
		expect(err).toHaveProperty('thrown');
	});

	it('rename validates name and calls backend', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const bad = await actions.rename(event({ cookies, params: { orgId: 'org1' }, form: { name: '' } }));
		expect(bad).toMatchObject({ status: 400 });

		const mockFetch = vi.fn(async () => json({ ok: true }));
		vi.stubGlobal('fetch', mockFetch);
		const ok = await actions.rename(event({ cookies, params: { orgId: 'org1' }, form: { name: 'New Name' } }));
		expect(ok).toMatchObject({ renamed: true });
	});

	it('invite validates email and handles 409 and success', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		const bad = await actions.invite(event({ cookies, params: { orgId: 'org1' }, form: { email: 'bad-email' } }));
		expect(bad).toMatchObject({ status: 400 });

		vi.stubGlobal('fetch', vi.fn(async () => json({}, 409)));
		const conflict = await actions.invite(event({ cookies, params: { orgId: 'org1' }, form: { email: 'test@example.com' } }));
		expect(conflict).toMatchObject({ status: 409 });

		vi.stubGlobal('fetch', vi.fn(async () => json({ ok: true })));
		const ok = await actions.invite(event({ cookies, params: { orgId: 'org1' }, form: { email: 'new@example.com' } }));
		expect(ok).toMatchObject({ invited: 'new@example.com' });
	});

	it('revoke, role, remove, and leave call backend', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		vi.stubGlobal('fetch', vi.fn(async () => json(orgData)));

		const revoked = await actions.revoke(event({ cookies, params: { orgId: 'org1' }, form: { code: 'c1' } }));
		expect(revoked).toMatchObject({ revoked: true });

		const roleChanged = await actions.role(event({ cookies, params: { orgId: 'org1' }, form: { user_id: 'u2', role: 'admin' } }));
		expect(roleChanged).toMatchObject({ changed: true });

		const removed = await actions.remove(event({ cookies, params: { orgId: 'org1' }, form: { user_id: 'u2' } }));
		expect(removed).toMatchObject({ removed: true });

		const left = await settle(() => actions.leave(event({ cookies, params: { orgId: 'org1' } })));
		expect(left).toHaveProperty('thrown');
	});

	it('resends an invite, makes a link, and deletes the workspace', async () => {
		const cookies = cookieJar({ access_token: 'valid' });
		vi.stubGlobal('fetch', vi.fn(async () => json({ code: 'c1', email: 'bob@example.com', link: false })));
		expect(await actions.resend(event({ cookies, params: { orgId: 'org1' }, form: { code: 'c1' } }))).toEqual({ resent: 'bob@example.com' });
		vi.stubGlobal('fetch', vi.fn(async () => json({ code: 'L1', email: null, link: true }, 201)));
		expect(await actions.link(event({ cookies, params: { orgId: 'org1' }, form: { role: 'member' } }))).toEqual({ linkCode: 'L1' });
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 404)));
		expect(await actions.resend(event({ cookies, params: { orgId: 'org1' }, form: { code: 'c1' } }))).toMatchObject({ status: 404 });
		expect(await actions.link(event({ cookies, params: { orgId: 'org1' }, form: {} }))).toMatchObject({ status: 404 });
		expect(await actions.delete(event({ cookies, params: { orgId: 'org1' } }))).toMatchObject({ status: 404 });

		vi.stubGlobal('fetch', vi.fn(async () => new Response('move or delete its projects first', { status: 409 })));
		expect(await actions.delete(event({ cookies, params: { orgId: 'org1' } }))).toMatchObject({ status: 409, data: { error: expect.any(String) } });
		vi.stubGlobal('fetch', vi.fn(async () => new Response('only owners can delete a workspace', { status: 403 })));
		expect(await actions.delete(event({ cookies, params: { orgId: 'org1' } }))).toMatchObject({ status: 403 });
		vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 204 })));
		expect(await settle(() => actions.delete(event({ cookies, params: { orgId: 'org1' } })))).toMatchObject({ thrown: { location: '/organizations' } });
	});
});
