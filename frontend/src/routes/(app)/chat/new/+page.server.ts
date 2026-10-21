import { fail, isRedirect, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { loadModelOptions, manageModelActions, saveSessionModel } from '$lib/server/modelOptions';
import { saveThinkingLevel } from '$lib/server/thinking';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

// A new chat before anything is said: nothing exists yet, so leaving here leaves nothing behind.
// The model picker starts on the person's default.
export const load: PageServerLoad = async ({ cookies, fetch }) => ({ title: m.nav_new_chat(), modelOptions: await loadModelOptions(fetch, cookies) });

export const actions: Actions = {
	// The first message creates the chat (with the thinking level and model picked here), then opens it.
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

		// Sent only when it isn't the person's default. Like the thinking level, a model that
		// can't be set leaves the chat on the default rather than stopping the message.
		const model = data.get('model');
		if (typeof model === 'string' && model) {
			const choice = new FormData();
			choice.set('key', model);
			try {
				await saveSessionModel(fetch, cookies, session_id, choice);
			} catch (e) {
				if (isRedirect(e)) throw e;
			}
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

	// Manage models, from the model picker: which models it offers, and connecting a provider.
	...manageModelActions,
};
