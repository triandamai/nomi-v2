import { apiFetch } from '$lib/server/api';
import type { ProjectSummary } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/projects');
	const projects: ProjectSummary[] = response.ok ? await response.json() : [];
	return { projects };
};
