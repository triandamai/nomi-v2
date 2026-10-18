// The app's snackbars, one at a time (the app layout shows them): a notification that arrives,
// a chat that finished while the person was elsewhere (see userActivity.ts).

export interface SnackbarItem {
	message: string;
	actionLabel: string;
	/** Where the action goes. */
	href: string;
	/** ms before it goes by itself. */
	timeout: number;
}

export const snackbars = $state<{ queue: SnackbarItem[] }>({ queue: [] });

export function showSnackbar(item: SnackbarItem) {
	snackbars.queue = [...snackbars.queue, item];
}

/** The next one to show, taken off the queue; ones pointing at the page already open are dropped. */
export function nextSnackbar(currentPath: string): SnackbarItem | null {
	while (snackbars.queue.length > 0) {
		const [next, ...rest] = snackbars.queue;
		snackbars.queue = rest;
		if (next.href !== currentPath) return next;
	}
	return null;
}
