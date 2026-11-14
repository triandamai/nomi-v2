import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, event, json } from '../../../test-stubs/server';
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
});
