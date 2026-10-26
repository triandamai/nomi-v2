import { toastFor, type ChatOutcome, type Waiting } from '$lib/chatActivity';
import { showSnackbar } from '$lib/snackbars.svelte';

// The person's own live feed (MQTT `user/{id}/activity`, relayed at /activity/ws). The app keeps
// it open while it's open, beside the open chat's own connection, so whatever the crew finishes
// anywhere reaches it: a chat left mid-work, a hand-off finishing later, a project in another
// tab. Each event becomes a snackbar with a way back, unless that chat is already on screen.
// Chat lists (the drawer's recent chats) also hear when a chat moved on, here or on another of
// the person's devices, so they stay in order without a reload.

/** One chat the crew is done with for now (backend: nomi_turn::activity). */
export interface ChatActivity {
	kind: 'ChatActivity';
	session_id: string;
	title: string | null;
	project_id: string | null;
	outcome: 'done' | 'failed' | 'waiting';
	waiting: Waiting | null;
	agent: string | null;
}

/** Something was said in one of the person's chats from one of their devices (backend:
 * nomi_realtime::UserEnvelope::ChatUpdated). */
export interface ChatUpdated {
	kind: 'ChatUpdated';
	session_id: string;
	project_id: string | null;
	title: string | null;
}

type UserEvent = ChatActivity | ChatUpdated;

const chatsChanged = new Set<(sessionId: string) => void>();

/** Calls `listener` with a chat's id whenever that chat moved on; returns the function that stops it. */
export function onChatsChanged(listener: (sessionId: string) => void): () => void {
	chatsChanged.add(listener);
	return () => chatsChanged.delete(listener);
}

/** Signed out or not allowed: reconnecting won't help. */
const TERMINAL_CLOSE_CODES = new Set([4401, 4403, 4404]);
const MAX_RETRY_DELAY_MS = 30_000;

/** Where a chat lives: a project's chat, or a regular one. */
export function chatHref(event: Pick<ChatActivity, 'session_id' | 'project_id'>): string {
	return event.project_id ? `/projects/session/${event.session_id}` : `/chat/${event.session_id}`;
}

function outcomeOf(event: ChatActivity): ChatOutcome {
	if (event.outcome === 'waiting' && event.waiting) return { kind: 'waiting', on: event.waiting };
	return event.outcome === 'failed' ? { kind: 'failed' } : { kind: 'done' };
}

/** What to do with one event from the feed. */
export function handleActivity(event: UserEvent) {
	if (event.kind === 'ChatUpdated' || event.kind === 'ChatActivity') {
		for (const listener of chatsChanged) listener(event.session_id);
	}
	if (event.kind !== 'ChatActivity') return;
	const href = chatHref(event);
	// Already looking at it, or this tab is in the background (the tab in front shows it).
	if (document.hidden || location.pathname === href) return;
	showSnackbar(toastFor(outcomeOf(event), { title: event.title ?? '', href }, event.agent));
}

/** Opens the feed (reconnecting when it drops); returns the function that closes it. */
export function connectUserActivity(): () => void {
	if (typeof WebSocket === 'undefined') return () => {};
	let socket: WebSocket | null = null;
	let closed = false;
	let retryDelay = 1000;
	let retry: ReturnType<typeof setTimeout> | undefined;

	function connect() {
		socket = new WebSocket('/activity/ws');
		socket.addEventListener('open', () => (retryDelay = 1000));
		socket.addEventListener('message', (message: MessageEvent<string>) => {
			try {
				handleActivity(JSON.parse(message.data) as UserEvent);
			} catch {
				// Not an event this build knows.
			}
		});
		socket.addEventListener('close', (event: CloseEvent) => {
			if (closed || TERMINAL_CLOSE_CODES.has(event.code)) return;
			retry = setTimeout(connect, retryDelay + Math.random() * 250);
			retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
		});
	}
	connect();

	return () => {
		closed = true;
		clearTimeout(retry);
		socket?.close();
	};
}
