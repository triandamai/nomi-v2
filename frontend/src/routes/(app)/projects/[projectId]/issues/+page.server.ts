import { error, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProjectDetail } from '$lib/types';
import type { PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

// Issues live in a sheet inside the project now, so the project keeps running while they're open.
export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
	const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}`);
	if (!response.ok) throw error(response.status === 404 ? 404 : 500, m.err_load_project());
	const project = (await response.json()) as ProjectDetail;
	throw redirect(303, `/projects/session/${project.session_id}?sheet=issues`);
};
