import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { SessionSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/sessions');
	if (!response.ok) {
		throw error(500, 'Failed to load chats.');
	}
	const { sessions } = (await response.json()) as { sessions: SessionSummary[] };
	return { sessions };
};

export const actions: Actions = {
	deleteSession: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const sessionId = data.get('sessionId');
		if (typeof sessionId !== 'string' || !sessionId) {
			return fail(400, { error: 'Invalid chat.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}`, { method: 'DELETE' });
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to delete chat.' });
		}

		return { success: true };
	},
};
