import { error, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProjectDetail } from '$lib/types';
import type { PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

// A link into a project by its id (from a notification, say), opening one of its sheets.
export const load: PageServerLoad = async ({ cookies, fetch, params, url }) => {
	const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}`);
	if (!response.ok) throw error(response.status === 404 ? 404 : 500, m.err_load_project());
	const project = (await response.json()) as ProjectDetail;
	const sheet = url.searchParams.get('sheet');
	throw redirect(303, `/projects/session/${project.session_id}${sheet ? `?sheet=${encodeURIComponent(sheet)}` : ''}`);
};
