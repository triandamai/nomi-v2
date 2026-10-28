// The unread count on the bell (sidebar and mobile header). The app layout seeds it; it then
// refreshes every minute while the tab is visible, and right after the inbox marks things read.

const POLL_MS = 60_000;

export interface LatestNotification {
	id: string;
	title: string;
	link: string | null;
	/** What happened, when it came live (the unread poll sends the title only). */
	body?: string;
}

export const inbox = $state<{ unread: number; latest: LatestNotification | null; arrived: LatestNotification | null }>({
	unread: 0,
	latest: null,
	/** One that came in while the app was open, to pop up as a snackbar. */
	arrived: null,
});

let seen = false;

export async function refreshUnread(): Promise<void> {
	try {
		const response = await fetch('/notifications/unread');
		if (!response.ok) return;
		const body = (await response.json()) as { unread: number; latest?: LatestNotification | null };
		const latest = body.latest ?? null;
		// Only something new since the last look counts as arriving, not what was already waiting.
		if (seen && latest && latest.id !== inbox.latest?.id && body.unread > inbox.unread) inbox.arrived = latest;
		inbox.unread = body.unread;
		inbox.latest = latest;
		seen = true;
	} catch {
		// Offline: keep the last count.
	}
}

/** A notification the live feed brought: counted, and popped up straight away. */
export function notificationArrived(notification: LatestNotification): void {
	if (inbox.latest?.id === notification.id) return;
	inbox.unread += 1;
	inbox.latest = notification;
	inbox.arrived = notification;
}

/** Starts polling; returns the function that stops it. */
export function pollUnread(): () => void {
	void refreshUnread();
	const timer = setInterval(() => {
		if (document.visibilityState === 'visible') void refreshUnread();
	}, POLL_MS);
	const onVisible = () => document.visibilityState === 'visible' && void refreshUnread();
	document.addEventListener('visibilitychange', onVisible);
	return () => {
		clearInterval(timer);
		document.removeEventListener('visibilitychange', onVisible);
	};
}

/** "99+" past 99. */
export function badgeLabel(count: number): string {
	return count > 99 ? '99+' : String(count);
}
