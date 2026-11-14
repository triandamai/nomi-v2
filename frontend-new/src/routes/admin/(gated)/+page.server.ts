import type { Dashboard } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** The overview: `GET /api/admin/dashboard` (dashboard: view). */
export const load: PageServerLoad = async (event) => ({
	dashboard: mustLoad(await getJson<Dashboard>(event, '/api/admin/dashboard')),
});
