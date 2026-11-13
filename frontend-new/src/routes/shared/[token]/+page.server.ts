import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { PageServerLoad } from './$types';

// A shared link (Flow / Chat — Shared link b8Flw8): the chat or reply someone shared, read-only,
// masked if they chose to, open to anyone or only to people signed in. An expired, turned-off or
// unknown link is a 404 from the backend alike (so links can't be probed); one for signed-in people
// only is a 401 for someone who isn't.

type SharedChat = {
	title: string | null;
	scope: 'chat' | 'reply';
	masked: boolean;
	expires_at: string | null;
	messages: { sender: 'user' | 'assistant'; agent_display_name: string | null; content: string; created_at: string }[];
};

export const load: PageServerLoad = async (event) => {
	let response: Response;
	try {
		response = await apiFetch(event, `/api/shared/${encodeURIComponent(event.params.token)}`, undefined, { signedOut: 'return' });
	} catch {
		error(503, { message: 'unreachable' });
	}
	if (response.status === 404) return { state: 'gone' as const };
	if (response.status === 401) return { state: 'signin' as const };
	if (!response.ok) error(response.status >= 500 ? 503 : response.status, { message: 'shared_failed' });
	const shared = (await response.json()) as SharedChat;
	const messages = await Promise.all(
		shared.messages.map(async (message) => ({ sender: message.sender, by: message.agent_display_name, at: message.created_at, html: await renderMarkdown(message.content) })),
	);
	return { state: 'open' as const, chat: { title: shared.title, scope: shared.scope, masked: shared.masked, expiresAt: shared.expires_at, messages } };
};
