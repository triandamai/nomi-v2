// The unread count on the bell (sidebar and mobile header). The app layout seeds it; it then
// refreshes every minute while the tab is visible, and right after the inbox marks things read.

const POLL_MS = 60_000;

export const inbox = $state({ unread: 0 });

export async function refreshUnread(): Promise<void> {
	try {
		const response = await fetch('/notifications/unread');
		if (response.ok) inbox.unread = ((await response.json()) as { unread: number }).unread;
	} catch {
		// Offline: keep the last count.
	}
}

/** Starts polling; returns the function that stops it. */
export function pollUnread(): () => void {
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
