import { fail, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { ThinkingLevel } from '$lib/types';
import { m } from '$lib/paraglide/messages';

const LEVELS = ['off', 'low', 'medium', 'high'];

/** The chat's thinking level; null (the picker hides) if it can't be read. */
export async function loadThinkingLevel(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string): Promise<ThinkingLevel | null> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/thinking`);
	if (!response.ok) return null;
	const { level } = (await response.json()) as { level: string };
	return LEVELS.includes(level) ? (level as ThinkingLevel) : null;
}

/** Shared `setThinking` form action body for the chat pages. */
export async function saveThinkingLevel(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string, data: FormData) {
	const level = String(data.get('level') ?? '');
	if (!LEVELS.includes(level)) return fail(400, { thinkingError: m.err_thinking_level() });
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/thinking`, { method: 'PUT', body: JSON.stringify({ level }) });
	if (!response.ok) return fail(response.status, { thinkingError: m.err_thinking_change() });
	return { thinkingLevel: level };
}
