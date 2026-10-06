import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { saveThinkingLevel } from '$lib/server/thinking';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

// A new chat before anything is said: nothing exists yet, so leaving here leaves nothing behind.
export const load: PageServerLoad = async () => ({ title: m.nav_new_chat() });

export const actions: Actions = {
	// The first message creates the chat (with the thinking level picked here), then opens it.
	sendMessage: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');
		if (typeof text !== 'string' || !text.trim()) {
			return fail(400, { error: m.err_empty_message() });
		}

		const created = await apiFetch(fetch, cookies, '/api/sessions', { method: 'POST' });
		if (!created.ok) {
			return fail(500, { error: m.home_new_chat_failed() });
		}
		const { session_id } = (await created.json()) as { session_id: string };

		const thinking = data.get('thinking');
		if (typeof thinking === 'string' && thinking) {
			const level = new FormData();
			level.set('level', thinking);
			await saveThinkingLevel(fetch, cookies, session_id, level);
		}

		const sent = await apiFetch(fetch, cookies, `/api/sessions/${session_id}/messages`, {
			method: 'POST',
			body: JSON.stringify({ text }),
		});
		if (!sent.ok) {
			// The chat exists; land there so the message can simply be sent again.
			throw redirect(303, `/chat/${session_id}`);
		}
		throw redirect(303, `/chat/${session_id}`);
	},
};
