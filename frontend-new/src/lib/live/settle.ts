// A reload after live events, once they've gone quiet, and never while the app is moving to
// another page: SvelteKit's invalidate() during a navigation reloads the page being left and drops
// the navigation (a new chat from Home stayed on Home when its own "chat changed" event landed).

export type SettledRefresh = { schedule: () => void; cancel: () => void };

export function settledRefresh(run: () => void, navigating: () => boolean, { wait = 400, retry = 100 }: { wait?: number; retry?: number } = {}): SettledRefresh {
	let timer: ReturnType<typeof setTimeout> | undefined;
	const attempt = () => {
		if (navigating()) {
			timer = setTimeout(attempt, retry);
			return;
		}
		timer = undefined;
		run();
	};
	return {
		schedule() {
			clearTimeout(timer);
			timer = setTimeout(attempt, wait);
		},
		cancel() {
			clearTimeout(timer);
			timer = undefined;
		},
	};
}
