import { fail, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';

type Change = { pinned: boolean } | { in_project_id: string | null };

async function update(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, change: Change) {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}`, { method: 'PATCH', body: JSON.stringify(change) });
	if (!response.ok) return fail(response.status, { updateError: m.chat_update_failed() });
	return { updated: (await response.json()) as { pinned: boolean; in_project_id: string | null } };
}

/** Pins or unpins a chat from a form post (`pinned` is "true" or "false"). */
export function pinSession(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, data: FormData) {
	return update(fetch, cookies, sessionId, { pinned: data.get('pinned') === 'true' });
}

/** Moves a chat into a project (`projectId`), or out of one when it's empty. */
export function moveSession(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, data: FormData) {
	const projectId = String(data.get('projectId') ?? '').trim();
	return update(fetch, cookies, sessionId, { in_project_id: projectId || null });
}
