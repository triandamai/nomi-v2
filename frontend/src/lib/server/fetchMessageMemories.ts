import { error, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { UsedMemory } from '$lib/types';

/** The memories one reply drew on, for its "Used N memories" sheet. */
export async function fetchMessageMemories(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, messageId: string): Promise<UsedMemory[]> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/messages/${messageId}/memories`);
	if (!response.ok) throw error(response.status, m.err_load_message());
	return ((await response.json()) as { memories: UsedMemory[] }).memories;
}
