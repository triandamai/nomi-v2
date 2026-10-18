import { m } from '$lib/paraglide/messages';
import type { SnackbarItem } from '$lib/snackbars.svelte';

// What to tell someone about a chat the crew finished while they were elsewhere in the app (see
// userActivity.ts): that it's done, failed, or is waiting on them for something.

/** What the crew left waiting on the person (backend: nomi_turn::activity). */
export type Waiting = 'question' | 'approval' | 'connect';

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
