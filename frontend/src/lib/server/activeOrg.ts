import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { setSession } from '$lib/server/sessionCookies';

/**
 * The workspace an access token works in (its `active_org_id`), read from the token's payload.
 * Only for showing which one is active: the API checks the token itself.
 */
export function activeOrgId(token: string | undefined | null): string | null {
	const payload = token?.split('.')[1];
	if (!payload) return null;
	try {
		const json = JSON.parse(Buffer.from(payload.replace(/-/g, '+').replace(/_/g, '/'), 'base64').toString('utf8')) as {
			active_org_id?: unknown;
		};
		return typeof json.active_org_id === 'string' ? json.active_org_id : null;
	} catch {
		return null;
	}
}

/**
 * Switches the person's active workspace (`PUT /api/auth/active-org`) and keeps the new access
 * token the way sign-in does. `false` when the API said no (not a member, or it failed).
 */
export async function switchActiveOrg(fetchFn: typeof fetch, cookies: Cookies, orgId: string): Promise<boolean> {
	const response = await apiFetch(fetchFn, cookies, '/api/auth/active-org', { method: 'PUT', body: JSON.stringify({ org_id: orgId }) });
	if (!response.ok) return false;
	const { access_token } = (await response.json()) as { access_token: string };
	setSession(cookies, { access_token });
	return true;
}
