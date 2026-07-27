import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	newChat: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/sessions', { method: 'POST' });
		if (!response.ok) {
			return fail(500, { error: 'Could not start a new chat.' });
		}
		const { session_id } = (await response.json()) as { session_id: string };
		throw redirect(303, `/chat/${session_id}`);
	},
};
