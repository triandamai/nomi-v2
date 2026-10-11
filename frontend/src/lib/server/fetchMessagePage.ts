import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { MessageItem, RenderedMessage } from '$lib/types';

/** How many messages a chat opens with (and loads per "earlier" page). */
export const MESSAGE_PAGE_SIZE = 50;

/**
 * A page of a chat's messages, oldest first and server-rendered, ending just before `before`
 * (or at the newest). The backend never starts a page inside a thread: it reaches back to the
 * person's message that started it. `null` when the chat can't be read.
 */
export async function fetchMessagePage(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	sessionId: string,
	before: string | null = null,
): Promise<{ status: number; messages: RenderedMessage[]; hasOlder: boolean } | { status: number; messages: null; hasOlder: false }> {
	const query = new URLSearchParams({ limit: String(MESSAGE_PAGE_SIZE) });
	if (before) query.set('before', before);
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/messages?${query}`);
	if (!response.ok) return { status: response.status, messages: null, hasOlder: false };
	const body = (await response.json()) as { messages: MessageItem[]; has_more?: boolean };
	const messages = await Promise.all(body.messages.map(async (message) => ({ ...message, content_html: await renderMarkdown(message.content) })));
	return { status: response.status, messages, hasOlder: Boolean(body.has_more) };
}
