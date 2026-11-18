import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import { kindLabel, splitAttachments } from '$lib/chat/attachments';
import { looksLikePhone } from '$lib/phone';
import type { PageServerLoad } from './$types';

// A shared link (Flow / Chat — Shared link b8Flw8): the chat or reply someone shared, read-only,
// masked if they chose to, open to anyone or only to people signed in. An expired, turned-off or
// unknown link is a 404 from the backend alike (so links can't be probed); one for signed-in people
// only is a 401 for someone who isn't.
// Files (UX contract audit-chat-files, G16, Link D5): a shared link never gives a file's address,
// so each file a message went with is named from its reference tag, and nothing more (its id and
// size stay here); the page shows it locked, `<type> · private`.

type SharedChat = {
	title: string | null;
	scope: 'chat' | 'reply';
	masked: boolean;
	expires_at: string | null;
	/** Who shared it (their initials when amounts and names are hidden). */
	shared_by?: string;
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
		shared.messages.map(async (message) => {
			const { text, files } = splitAttachments(message.content);
			return { sender: message.sender, by: message.agent_display_name, at: message.created_at, html: await renderMarkdown(text), files: files.map((file) => ({ name: file.name, type: kindLabel(file) })) };
		}),
	);
	const phone = looksLikePhone(event.request?.headers.get('user-agent') ?? '', event.request?.headers.get('sec-ch-ua-mobile'));
	return {
		state: 'open' as const,
		phone,
		chat: { title: shared.title, scope: shared.scope, masked: shared.masked, expiresAt: shared.expires_at, sharedBy: shared.shared_by ?? '', files: messages.some((message) => message.files.length > 0), messages },
	};
};
