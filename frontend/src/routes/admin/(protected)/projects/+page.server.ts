import { apiFetch } from '$lib/server/api';
import type { AdminProject } from '$lib/adminProjects';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const q = url.searchParams.get('q')?.trim() ?? '';
	const response = await apiFetch(fetch, cookies, `/api/admin/projects${q ? `?q=${encodeURIComponent(q)}` : ''}`);
	return { q, projects: response.ok ? ((await response.json()) as { projects: AdminProject[] }).projects : null };
};
