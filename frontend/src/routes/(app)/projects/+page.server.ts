import { error, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProjectSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/projects');
	if (!response.ok) {
		throw error(500, 'Failed to load projects.');
	}
	const projects: ProjectSummary[] = await response.json();
	return { projects };
};

export const actions: Actions = {
	// Creates the project row up front, in the same request as the chat session — the workspace
	// page (keyed by this session id) recognizes it as a project immediately, rather than
	// depending on the planning agent reliably calling its own create_project tool once the
	// conversation gets going (that dependency turned out to be unreliable in practice).
	createProject: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/projects', { method: 'POST' });
		if (!response.ok) {
			throw error(500, 'Could not start a new project.');
		}
		const { session_id } = (await response.json()) as { session_id: string; project_id: string };
		throw redirect(303, `/projects/session/${session_id}`);
	},
};
