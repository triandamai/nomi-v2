import type { AdminProject } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** Every project (projects: view), searched by name or owner with `?query=`. */
export const load: PageServerLoad = async (event) => {
	const query = (event.url.searchParams.get('query') ?? '').trim();
	const loaded = mustLoad(await getJson<{ projects: AdminProject[] }>(event, `/api/admin/projects${query ? `?q=${encodeURIComponent(query)}` : ''}`));
	return { query, projects: loaded.ok ? { ok: true as const, data: loaded.data?.projects ?? [] } : loaded };
};
