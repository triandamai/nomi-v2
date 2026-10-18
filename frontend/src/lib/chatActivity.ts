import { m } from '$lib/paraglide/messages';
import type { ContentBlock, MessageItem } from '$lib/types';
import type { SnackbarItem } from '$lib/snackbars.svelte';

// What to tell someone about a chat they left while the crew was still working (see
// backgroundChats.ts): that it finished, failed, or is waiting on them for something.

/** What the crew left waiting on the person, from the newest message that needs them. */
export type Waiting = 'question' | 'approval' | 'connect';

function waitingOn(block: ContentBlock): Waiting | null {
	switch (block.kind) {
		case 'questions':
			return 'question';
		case 'approval_request':
			return block.status === 'pending' ? 'approval' : null;
		case 'workspace_connect':
		case 'connector_connect':
			return 'connect';
		case 'view':
			// A decision or a form asks for an answer; other views are just shown.
			return block.view === 'decision' || block.view === 'form' ? 'question' : null;
		default:
			return null;
	}
}

/** What the crew is waiting on in these messages (newest first wins), or null if nothing. */
export function waitingIn(messages: Pick<MessageItem, 'sender' | 'content_blocks'>[]): Waiting | null {
	for (let i = messages.length - 1; i >= 0; i--) {
		const message = messages[i];
		if (message.sender !== 'assistant') continue;
		for (const block of message.content_blocks ?? []) {
			const waiting = waitingOn(block);
			if (waiting) return waiting;
		}
	}
	return null;
}

export type ChatOutcome = { kind: 'done' } | { kind: 'failed' } | { kind: 'waiting'; on: Waiting };

/** A snackbar about a chat; its action goes to the chat or project the work was in. Something
 * waiting on the person stays up longer. */
export type ChatToast = SnackbarItem;

/** The snackbar for a chat's outcome. `agent` is who answered last (Nomi when unknown). */
export function toastFor(outcome: ChatOutcome, chat: { title: string; href: string }, agent: string | null): ChatToast {
	const who = agent || 'Nomi';
	const title = chat.title.trim() || m.chat_title();
	switch (outcome.kind) {
		case 'failed':
			return { message: m.away_failed({ title }), actionLabel: m.away_open(), href: chat.href, timeout: 8000 };
		case 'done':
			return { message: m.away_done({ who, title }), actionLabel: m.away_open(), href: chat.href, timeout: 6000 };
		case 'waiting': {
			const [message, actionLabel] =
				outcome.on === 'question'
					? [m.away_question({ who, title }), m.away_answer()]
					: outcome.on === 'approval'
						? [m.away_approval({ who, title }), m.away_review()]
						: [m.away_connect({ who, title }), m.away_open()];
			return { message, actionLabel, href: chat.href, timeout: 12000 };
		}
	}
}
