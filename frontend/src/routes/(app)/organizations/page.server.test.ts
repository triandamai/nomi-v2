import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json, settle } from '../../../test-stubs/server';
import { actions, load } from './+page.server';

afterEach(() => vi.unstubAllGlobals());

const ORG = '22222222-2222-2222-2222-222222222222';
const tokenFor = (org: string) => `h.${Buffer.from(JSON.stringify({ active_org_id: org })).toString('base64url')}.s`;

describe('organizations', () => {
	it('says which workspace the token works in', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ orgs: [], project_org_id: null })));
		const data = await load(event({ cookies: cookieJar({ access_token: tokenFor(ORG) }) }));
		expect(data).toMatchObject({ activeOrgId: ORG });
	});

	it('switches the active workspace and keeps the new token', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ access_token: tokenFor(ORG) })));
		const cookies = cookieJar({ access_token: 'old' });
		expect(await actions.switch(event({ cookies, form: { org_id: ORG } }))).toEqual({ switched: ORG });
		expect(cookies.get('access_token')).toBe(tokenFor(ORG));
	});

	it('says so when the switch is refused', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => json({ error: 'not_a_member' }, 404)));
		expect(await actions.switch(event({ cookies: cookieJar({ access_token: 'old' }), form: { org_id: ORG } }))).toMatchObject({
			status: 400,
			data: { switchError: expect.any(String) },
		});
		expect(await actions.switch(event({ form: {} }))).toMatchObject({ status: 400 });
	});

	it('lists pending invites beside the workspaces, and gets by without them', async () => {
		const invite = { code: 'c1', org_id: ORG, org_name: 'Kopi Kita', inviter: 'Andi', role: 'member', projects: 2, expires_at: '', project: null };
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => (String(url).endsWith('/api/invites') ? json({ invites: [invite] }) : json({ orgs: [], project_org_id: null, open_org_id: ORG }))),
		);
		expect(await load(event())).toMatchObject({ invites: [invite], openOrgId: ORG });
		vi.stubGlobal(
			'fetch',
			vi.fn(async (url: string) => (String(url).endsWith('/api/invites') ? json({}, 500) : json({ orgs: [], project_org_id: null }))),
		);
		expect(await load(event())).toMatchObject({ invites: [], openOrgId: null });
	});

	it('joins an invite and opens the workspace, or declines it', async () => {
		const calls: string[] = [];
		vi.stubGlobal('fetch', vi.fn(async (url: string) => (calls.push(String(url)), json({ org_id: ORG }))));
		expect(await settle(() => actions.join(event({ form: { code: 'c1' } })))).toMatchObject({ thrown: { location: `/organizations/${ORG}` } });
		expect(await actions.decline(event({ form: { code: 'c1' } }))).toEqual({ declined: 'c1' });
		expect(calls.some((url) => url.endsWith('/api/invites/c1/accept'))).toBe(true);
		expect(calls.some((url) => url.endsWith('/api/invites/c1/decline'))).toBe(true);
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 410)));
		expect(await actions.join(event({ form: { code: 'c1' } }))).toMatchObject({ status: 410 });
		expect(await actions.decline(event({ form: { code: 'c1' } }))).toMatchObject({ status: 410 });
		expect(await actions.join(event({ form: {} }))).toMatchObject({ status: 400 });
		expect(await actions.decline(event({ form: {} }))).toMatchObject({ status: 400 });
	});

	it('saves where Nomi opens, last used as null', async () => {
		const bodies: unknown[] = [];
		vi.stubGlobal('fetch', vi.fn(async (_url: string, init?: RequestInit) => (bodies.push(JSON.parse(String(init?.body))), new Response(null, { status: 204 }))));
		expect(await actions.openIn(event({ form: { org_id: '' } }))).toEqual({ openSaved: true });
		expect(await actions.openIn(event({ form: { org_id: ORG } }))).toEqual({ openSaved: true });
		expect(bodies).toEqual([{ org_id: null }, { org_id: ORG }]);
		vi.stubGlobal('fetch', vi.fn(async () => json({}, 404)));
		expect(await actions.openIn(event({ form: { org_id: ORG } }))).toMatchObject({ status: 404 });
	});
});
