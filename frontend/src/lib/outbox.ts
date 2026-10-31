// A message that couldn't go out (offline, or the sign-in ran out) waits on this device, one per
// chat, and the chat sends it once it can: back online, or signed in again. As frontend-new's
// Banner and SessionExpired put it, nothing typed is lost.

const PREFIX = 'nomi:outbox:';

/** Keeps `text` (as it would be posted, file references included) for the chat at `chat`. */
export function keep(chat: string, text: string): void {
	try {
		localStorage.setItem(PREFIX + chat, text);
	} catch {
		// Private mode or storage off: nothing to keep it in.
	}
}

/** Whether a message waits for `chat`. */
export function waiting(chat: string): boolean {
	try {
		return localStorage.getItem(PREFIX + chat) !== null;
	} catch {
		return false;
	}
}

/** The message waiting for `chat`, taken out of the outbox; null when there's none. */
export function take(chat: string): string | null {
	try {
		const text = localStorage.getItem(PREFIX + chat);
		if (text !== null) localStorage.removeItem(PREFIX + chat);
		return text;
	} catch {
		return null;
	}
}
