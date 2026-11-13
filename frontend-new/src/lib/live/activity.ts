import { createLiveLink, type LinkSocket } from './link';

// The person's own live feed (MQTT `user/{id}/activity`, relayed at /activity/ws), open while the
// app is: a chat that moved on anywhere (here or on another of their devices), a new inbox item,
// and this device being signed out from another. It is also this tab's presence: the backend names
// the broker connection `<deviceId>-<tabId>` and hears from it whether the tab is shown.
// Ported from the old app's userActivity (frontend/src/lib/userActivity.ts).

/** One chat the crew is done with for now (backend: nomi_turn::activity). */
type ChatActivity = { kind: 'ChatActivity'; session_id: string };
/** Something was said in one of the person's chats (nomi_realtime::UserEnvelope::ChatUpdated). */
type ChatUpdated = { kind: 'ChatUpdated'; session_id: string };
/** One of the person's chats was deleted, on any device (UserEnvelope::ChatDeleted). */
type ChatDeleted = { kind: 'ChatDeleted'; session_id: string };
/** A new item in the person's inbox (nomi_realtime::UserEvent::NotificationCreated). */
type NotificationCreated = { kind: 'NotificationCreated'; notification_id: string; title: string; body: string; link: string | null };
/** One of the person's devices was signed out from another (UserEnvelope::DeviceSignedOut). */
type DeviceSignedOut = { kind: 'DeviceSignedOut'; device_id: string };

export type ActivityEvent = ChatActivity | ChatUpdated | ChatDeleted | NotificationCreated | DeviceSignedOut;

export type Notification = { id: string; title: string; body: string; link: string | null };

export interface ActivityHandlers {
	/** A chat moved on, changed or was deleted: the chat lists are out of date. */
	chatsChanged(sessionId: string): void;
	notification(notification: Notification): void;
	/** This device was signed out from another: its sign-in is already over. */
	signedOutHere(): void;
	/** The feed came back after a drop: anything said meanwhile wasn't replayed. */
	resumed(): void;
	/** The server ended the feed for good (4401: the sign-in is over). */
	lost(code: number): void;
}

function isEvent(value: unknown): value is { kind: string } & Record<string, unknown> {
	return typeof value === 'object' && value !== null && typeof (value as { kind?: unknown }).kind === 'string';
}

/** What to do with one event from the feed; `deviceId` is this browser's. */
export function routeActivity(event: unknown, deviceId: string | null, on: Pick<ActivityHandlers, 'chatsChanged' | 'notification' | 'signedOutHere'>): void {
	if (!isEvent(event)) return;
	const known = event as ActivityEvent;
	switch (known.kind) {
		case 'ChatActivity':
		case 'ChatUpdated':
		case 'ChatDeleted':
			on.chatsChanged(known.session_id);
			return;
		case 'NotificationCreated':
			on.notification({ id: known.notification_id, title: known.title, body: known.body, link: known.link });
			return;
		case 'DeviceSignedOut':
			if (deviceId && known.device_id.toLowerCase() === deviceId.toLowerCase()) on.signedOutHere();
			return;
	}
}

/** Whether the tab is shown, and when that changes (the document, in the browser). */
export interface PageVisibility {
	hidden(): boolean;
	/** Returns the function that stops listening. */
	onVisibilityChange(listener: () => void): () => void;
}

const browserPage: PageVisibility = {
	hidden: () => document.hidden,
	onVisibilityChange(listener) {
		document.addEventListener('visibilitychange', listener);
		return () => document.removeEventListener('visibilitychange', listener);
	},
};

type Options = {
	deviceId: string | null;
	/** This tab's id ($lib/device tabId()). */
	tab: string;
	handlers: ActivityHandlers;
	page?: PageVisibility;
	openSocket?: (url: string) => LinkSocket;
};

/** Opens the feed (reconnecting when it drops); returns the function that closes it. */
export function connectActivity({ deviceId, tab, handlers, page = browserPage, openSocket = (url) => new WebSocket(url) }: Options): () => void {
	const url = `/activity/ws?tab=${encodeURIComponent(tab)}`;
	const presence = () => JSON.stringify({ op: 'presence', state: page.hidden() ? 'away' : 'online' });

	const link = createLiveLink(
		{
			opened(resumed) {
				// A new connection counts as shown; a hidden tab says so straight away.
				if (page.hidden()) link.send(presence());
				if (resumed) handlers.resumed();
			},
			message(data) {
				try {
					routeActivity(JSON.parse(String(data)), deviceId, handlers);
				} catch {
					// Not an event this build knows.
				}
			},
			lost: (code) => handlers.lost(code),
		},
		() => openSocket(url),
	);
	const stopWatching = page.onVisibilityChange(() => link.send(presence()));

	return () => {
		stopWatching();
		link.close();
	};
}
