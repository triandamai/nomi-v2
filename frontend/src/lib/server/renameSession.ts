import { fail, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';

/** Renames a chat from a form post (`title`); answers with the name as saved. */
export async function renameSession(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, data: FormData) {
	const title = String(data.get('title') ?? '').trim();
	if (!title) return fail(400, { renameError: m.chat_rename_empty() });
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}`, {
		method: 'PATCH',
		body: JSON.stringify({ title }),
	});
	if (!response.ok) return fail(response.status, { renameError: m.chat_rename_failed() });
	const saved = (await response.json()) as { title: string };
	return { renamed: saved.title };
}
