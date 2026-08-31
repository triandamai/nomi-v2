import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ProjectDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}`);
	if (!response.ok) {
		throw error(response.status === 404 ? 404 : 500, 'Failed to load project.');
	}
	const project: ProjectDetail = await response.json();
	return { project };
};

export const actions: Actions = {
	loadFile: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const path = data.get('path');
		if (typeof path !== 'string' || !path) {
			return fail(400, { error: 'Invalid path.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}/files/${path}`);
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to load file.' });
		}
		const content = await response.text();
		return { success: true, path, content };
	},

	saveFile: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const path = data.get('path');
		const content = data.get('content');
		if (typeof path !== 'string' || !path || typeof content !== 'string') {
			return fail(400, { error: 'Invalid file.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/projects/${params.projectId}/files/${path}`, {
			method: 'PUT',
			body: JSON.stringify({ content }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to save file.' });
		}
		return { success: true };
	},
};
