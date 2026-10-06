import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { HomeSummary, SessionSummary } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

const RECENT_LIMIT = 6;

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	// Best-effort: the home page still works as a launcher if either can't load.
	const [response, summaryResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/sessions'),
		apiFetch(fetch, cookies, '/api/home'),
	]);
	const sessions: SessionSummary[] = response.ok
		? ((await response.json()) as { sessions: SessionSummary[] }).sessions
		: [];
	const summary: HomeSummary | null = summaryResponse.ok ? ((await summaryResponse.json()) as HomeSummary) : null;
	return { recentSessions: sessions.slice(0, RECENT_LIMIT), summary };
};

export const actions: Actions = {
	// Starts a chat. When the home composer sent text along, it becomes the chat's first message
	// so asking from home is one step, not "new chat, then type again".
	newChat: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const text = data.get('text');

		const response = await apiFetch(fetch, cookies, '/api/sessions', { method: 'POST' });
		if (!response.ok) {
			return fail(500, { error: m.home_new_chat_failed() });
		}
		const { session_id } = (await response.json()) as { session_id: string };

		if (typeof text === 'string' && text.trim()) {
			const sent = await apiFetch(fetch, cookies, `/api/sessions/${session_id}/messages`, {
				method: 'POST',
				body: JSON.stringify({ text }),
			});
			if (!sent.ok) {
				// The chat exists; land there so the text can simply be resent.
				throw redirect(303, `/chat/${session_id}`);
			}
		}
		throw redirect(303, `/chat/${session_id}`);
	},
};
