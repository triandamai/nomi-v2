import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { SessionSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/sessions');
	if (!response.ok) {
		throw error(500, m.err_load_chats());
	}
	const { sessions } = (await response.json()) as { sessions: SessionSummary[] };
	return { sessions };
};

export const actions: Actions = {
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
