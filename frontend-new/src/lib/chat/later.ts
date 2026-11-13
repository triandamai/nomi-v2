// Something done a little later, with a way back until then: a deleted chat goes after the Undo
// toast's ten seconds (Manage chat & long threads, step 6: "Undo restores the whole chat for 10
// seconds"), since the backend deletes at once and for good. Leaving the page does what's still
// waiting straight away (`flush`, on pagehide), so nothing undone is left behind.

export function createLater() {
	const waiting = new Map<string, { run: () => void; timer: ReturnType<typeof setTimeout> }>();

	function finish(key: string) {
		const entry = waiting.get(key);
		if (!entry) return;
		clearTimeout(entry.timer);
		waiting.delete(key);
		entry.run();
	}

	return {
		schedule(key: string, run: () => void, ms: number) {
			finish(key);
			waiting.set(key, { run, timer: setTimeout(() => finish(key), ms) });
		},
		/** Takes it back; false when it already happened (or never was). */
		undo(key: string): boolean {
			const entry = waiting.get(key);
			if (!entry) return false;
			clearTimeout(entry.timer);
			waiting.delete(key);
			return true;
		},
		pending: (key: string) => waiting.has(key),
		flush() {
			for (const key of [...waiting.keys()]) finish(key);
		},
	};
}

/** The app's own, for chats deleted with Undo. */
export const later = createLater();
