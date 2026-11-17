import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { AdminIssue } from '$lib/adminProjects';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
	const response = await apiFetch(fetch, cookies, `/api/admin/projects/${encodeURIComponent(params.id)}/issues`);
	if (!response.ok) throw error(response.status, m.admin_projects_failed());
	return (await response.json()) as { project: { id: string; name: string }; issues: AdminIssue[] };
};
