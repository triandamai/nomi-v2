import { error } from '@sveltejs/kit';
import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { MessageItem, RenderedMessage } from '$lib/types';

/** Fetches one message from the backend and server-renders its markdown (skipped when it
 * carries content_blocks — those render client-side via ContentBlockView, not {@html}). Used by
 * the two `message/[messageId]/+server.ts` proxy routes (chat and project-workspace pages) so
 * ChatThread.svelte can patch a single message instead of refetching the whole conversation. */
export async function fetchRenderedMessage(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	sessionId: string,
	messageId: string,
): Promise<RenderedMessage> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/messages/${messageId}`);
	if (!response.ok) {
		throw error(response.status, 'Could not load this message.');
	}
	const message = (await response.json()) as MessageItem;
	const content_html =
		message.content_blocks && message.content_blocks.length > 0 ? '' : await renderMarkdown(message.content);
	return { ...message, content_html };
}
