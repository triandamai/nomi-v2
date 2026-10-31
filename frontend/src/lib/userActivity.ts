import { toastFor, type ChatOutcome, type Waiting } from '$lib/chatActivity';
import { notificationArrived } from '$lib/notifications.svelte';
import { showSnackbar } from '$lib/snackbars.svelte';
import { tabId } from '$lib/device';

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

/** One of the person's chats was deleted, on any device (backend: nomi_realtime::UserEnvelope::ChatDeleted). */
export interface ChatDeleted {
	kind: 'ChatDeleted';
	session_id: string;
}

/** One of the person's devices came on, went to the background or went off (backend:
 * nomi_realtime::UserEnvelope::DevicePresence). */
export interface DevicePresence {
	kind: 'DevicePresence';
	device_id: string;
	presence: 'online' | 'away' | 'offline';
	last_seen_at: string;
}

/** One of the person's devices was signed out from another (backend: nomi_realtime::UserEnvelope::DeviceSignedOut). */
export interface DeviceSignedOut {
	kind: 'DeviceSignedOut';
	device_id: string;
}

/** A new item in the person's inbox (backend: nomi_realtime::UserEvent::NotificationCreated). */
export interface NotificationCreated {
	kind: 'NotificationCreated';
	notification_id: string;
	category: string;
	title: string;
	body: string;
	link: string | null;
}

type UserEvent = ChatActivity | ChatUpdated | ChatDeleted | NotificationCreated | DevicePresence | DeviceSignedOut;

const chatsChanged = new Set<(sessionId: string) => void>();

/** Calls `listener` with a chat's id whenever that chat moved on or was deleted; returns the function that stops it. */
export function onChatsChanged(listener: (sessionId: string) => void): () => void {
	chatsChanged.add(listener);
	return () => chatsChanged.delete(listener);
}

const devicesChanged = new Set<(event: DevicePresence) => void>();

/** Calls `listener` whenever one of the person's devices comes on, goes to the background or goes off. */
export function onDevicePresence(listener: (event: DevicePresence) => void): () => void {
	devicesChanged.add(listener);
	return () => devicesChanged.delete(listener);
}

/** Signed out from another device: this browser's sign-in is already over, so leave. */
async function signOutHere() {
	await fetch('/logout', { method: 'POST' }).catch(() => undefined);
	location.assign('/login');
}

/** What this tab says about itself: online while it's shown, away while it's hidden. */
function presenceFrame(): string {
	return JSON.stringify({ op: 'presence', state: document.hidden ? 'away' : 'online' });
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

/** What to do with one event from the feed; `deviceId` is this browser's. */
export function handleActivity(event: UserEvent, deviceId: string | null = null) {
	if (event.kind === 'DevicePresence') {
		for (const listener of devicesChanged) listener(event);
		return;
	}
	if (event.kind === 'DeviceSignedOut') {
		if (deviceId && event.device_id.toLowerCase() === deviceId.toLowerCase()) void signOutHere();
		return;
	}
	if (event.kind === 'NotificationCreated') {
		notificationArrived({ id: event.notification_id, title: event.title, body: event.body, link: event.link });
		return;
	}
	if (event.kind === 'ChatUpdated' || event.kind === 'ChatActivity' || event.kind === 'ChatDeleted') {
		for (const listener of chatsChanged) listener(event.session_id);
	}
	if (event.kind !== 'ChatActivity') return;
	const href = chatHref(event);
	// Already looking at it, or this tab is in the background (the tab in front shows it).
	if (document.hidden || location.pathname === href) return;
	showSnackbar(toastFor(outcomeOf(event), { title: event.title ?? '', href }, event.agent));
}

/**
 * Opens the feed (reconnecting when it drops); returns the function that closes it. It is also
 * this tab's presence: the backend names its broker connection `<deviceId>-<tabId>` and hears
 * from it whether the tab is shown. `deviceId` is this browser's (the layout has it).
 */
export function connectUserActivity(deviceId: string | null = null): () => void {
	if (typeof WebSocket === 'undefined') return () => {};
	let socket: WebSocket | null = null;
	let closed = false;
	let retryDelay = 1000;
	let retry: ReturnType<typeof setTimeout> | undefined;
	const tab = tabId();

	// A newly opened socket counts as shown; a hidden tab says so straight away.
	function sayPresence() {
		if (socket?.readyState === WebSocket.OPEN) socket.send(presenceFrame());
	}
	const watchesVisibility = typeof document.addEventListener === 'function';
	if (watchesVisibility) document.addEventListener('visibilitychange', sayPresence);

	function connect() {
		socket = new WebSocket(`/activity/ws?tab=${tab}`);
		socket.addEventListener('open', () => {
			retryDelay = 1000;
			if (document.hidden) sayPresence();
		});
		socket.addEventListener('message', (message: MessageEvent<string>) => {
			try {
				handleActivity(JSON.parse(message.data) as UserEvent, deviceId);
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
		if (watchesVisibility) document.removeEventListener('visibilitychange', sayPresence);
		socket?.close();
	};
}
