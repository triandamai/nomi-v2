import { apiUrl } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { PageServerLoad } from './$types';

type SharedChat = {
	title: string | null;
	scope: 'reply' | 'chat';
	masked: boolean;
	expires_at: string | null;
	messages: { sender: 'user' | 'assistant'; agent_display_name: string | null; content: string; created_at: string }[];
};

// A shared reply or chat, read-only. Public when its owner said anyone may view; otherwise the
// reader's sign-in goes along, and without one the page asks for it.
export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const accessToken = cookies.get('access_token');
	const response = await fetch(apiUrl(`/api/shared/${encodeURIComponent(params.token)}`), {
		headers: accessToken ? { Authorization: `Bearer ${accessToken}` } : {},
	}).catch(() => null);
	if (response?.status === 401) return { state: 'sign_in' as const };
	if (!response?.ok) return { state: 'gone' as const };
	const chat = (await response.json()) as SharedChat;
	const messages = await Promise.all(chat.messages.map(async (message) => ({ ...message, html: await renderMarkdown(message.content) })));
	return { state: 'ready' as const, chat: { ...chat, messages } };
};
