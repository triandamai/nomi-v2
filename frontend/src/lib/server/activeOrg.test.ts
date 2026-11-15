import { afterEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, json } from '../../test-stubs/server';
import { activeOrgId, switchActiveOrg } from './activeOrg';

const ORG = '11111111-1111-1111-1111-111111111111';

function token(payload: unknown): string {
	const part = Buffer.from(JSON.stringify(payload)).toString('base64url');
	return `header.${part}.signature`;
}

afterEach(() => vi.unstubAllGlobals());

describe('activeOrgId', () => {
	it('reads the workspace from the token', () => {
		expect(activeOrgId(token({ sub: 'u', active_org_id: ORG }))).toBe(ORG);
	});

	it('is null for a missing or unreadable token', () => {
		expect(activeOrgId(undefined)).toBeNull();
		expect(activeOrgId('not-a-token')).toBeNull();
		expect(activeOrgId('a.%%%.c')).toBeNull();
		expect(activeOrgId(token({ sub: 'u' }))).toBeNull();
	});
});

describe('switchActiveOrg', () => {
	it('keeps the new access token', async () => {
		const fetchFn = vi.fn(async () => json({ access_token: 'new-token' }));
		const cookies = cookieJar({ access_token: 'old-token' });
		expect(await switchActiveOrg(fetchFn as never, cookies, ORG)).toBe(true);
		expect(cookies.get('access_token')).toBe('new-token');
		const [url, init] = fetchFn.mock.calls[0] as unknown as [string, RequestInit];
		expect(url).toMatch(/\/api\/auth\/active-org$/);
		expect(init.method).toBe('PUT');
		expect(JSON.parse(String(init.body))).toEqual({ org_id: ORG });
	});

	it('leaves the session alone when the API says no', async () => {
		const fetchFn = vi.fn(async () => json({ error: 'not_a_member' }, 404));
		const cookies = cookieJar({ access_token: 'old-token' });
		expect(await switchActiveOrg(fetchFn as never, cookies, ORG)).toBe(false);
		expect(cookies.get('access_token')).toBe('old-token');
	});
});
