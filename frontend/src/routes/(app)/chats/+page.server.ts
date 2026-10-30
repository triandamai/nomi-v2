import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renameSession } from '$lib/server/renameSession';
import { moveSession, pinSession } from '$lib/server/updateSession';
import type { ProjectList, SessionSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [response, projectsResponse] = await Promise.all([apiFetch(fetch, cookies, '/api/sessions'), apiFetch(fetch, cookies, '/api/projects')]);
	if (!response.ok) {
		throw error(500, m.err_load_chats());
	}
	const { sessions } = (await response.json()) as { sessions: SessionSummary[] };
	// The projects a chat can be moved into. Without them the list still works; Move just offers none.
	const projects = projectsResponse.ok ? ((await projectsResponse.json()) as ProjectList).projects.map(({ id, name }) => ({ id, name })) : [];
	return { sessions, projects };
};

/** The chat a form post is about, or null when the form doesn't say. */
function sessionIdOf(data: FormData): string | null {
	const sessionId = data.get('sessionId');
	return typeof sessionId === 'string' && sessionId ? sessionId : null;
}

export const actions: Actions = {
	renameSession: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const sessionId = data.get('sessionId');
		if (typeof sessionId !== 'string' || !sessionId) {
			return fail(400, { error: m.err_invalid_chat() });
		}
		return renameSession(fetch, cookies, sessionId, data);
	},
	pinSession: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const sessionId = sessionIdOf(data);
		if (!sessionId) return fail(400, { error: m.err_invalid_chat() });
		return pinSession(fetch, cookies, sessionId, data);
	},
	moveSession: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const sessionId = sessionIdOf(data);
		if (!sessionId) return fail(400, { error: m.err_invalid_chat() });
		return moveSession(fetch, cookies, sessionId, data);
	},
	deleteSession: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const sessionId = data.get('sessionId');
		if (typeof sessionId !== 'string' || !sessionId) {
			return fail(400, { error: m.err_invalid_chat() });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}`, { method: 'DELETE' });
		if (!response.ok) {
			return fail(response.status, { error: m.err_delete_chat() });
		}

		return { success: true };
	},
};
