import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Preferences } from '$lib/types';
import type { LayoutServerLoad } from './$types';

function hasAction(permissions: string[], resourcePrefix: string, action: string): boolean {
	return permissions.some((p) => {
		if (!p.startsWith(resourcePrefix)) return false;
		const match = p.match(/\[(.*)\]$/);
		return match ? match[1].split(',').includes(action) : false;
	});
}

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const response = await apiFetch(fetch, cookies, '/api/whoami');
	if (!response.ok) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const claims = (await response.json()) as { permissions: string[] };
	const isStaff = claims.permissions.some((permission) => permission.startsWith('nomi:admin:'));
	if (!isStaff) {
		throw redirect(303, '/?error=forbidden');
	}

	const canManageSystemConfig = hasAction(claims.permissions, 'nomi:admin:system_config:', 'manage');
	const canViewUsers = hasAction(claims.permissions, 'nomi:admin:user:', 'view');
	const canManageUsers = hasAction(claims.permissions, 'nomi:admin:user:', 'manage');

	const preferencesResponse = await apiFetch(fetch, cookies, '/api/preferences');
	const preferences: Preferences = preferencesResponse.ok
		? ((await preferencesResponse.json()) as Preferences)
		: { theme: 'system', accent_color: 'green' };

	return { canManageSystemConfig, canViewUsers, canManageUsers, preferences };
};
