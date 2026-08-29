import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const response = await apiFetch(fetch, cookies, '/api/whoami');
	if (!response.ok) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const claims = (await response.json()) as { permissions: string[] };
	const isSystemAdmin = claims.permissions.some((permission) =>
		permission.startsWith('nomi:admin:system_config:'),
	);
	if (!isSystemAdmin) {
		throw redirect(303, '/?error=forbidden');
	}

	return {};
};
