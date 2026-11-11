// A message that couldn't go out (offline, or the sign-in ran out) waits on this device, one per
// chat, and the chat sends it once it can: back online, or signed in again. As the Banner and
// SessionExpired put it, nothing typed is lost.
// Ported from the old app's outbox (frontend/src/lib/outbox.ts); the storage can be handed in.

const PREFIX = 'nomi:outbox:';

export type OutboxStorage = Pick<Storage, 'getItem' | 'setItem' | 'removeItem'>;

function browserStorage(): OutboxStorage | null {
	try {
		return globalThis.localStorage ?? null;
	} catch {
		return null;
	}
}

/** Keeps `text` (as it would be posted) for the chat at `chat`. */
export function keep(chat: string, text: string, storage = browserStorage()): void {
	try {
		storage?.setItem(PREFIX + chat, text);
	} catch {
		// Private mode or storage off: nothing to keep it in.
	}
}

/** Whether a message waits for `chat`. */
export function waiting(chat: string, storage = browserStorage()): boolean {
	try {
		return (storage?.getItem(PREFIX + chat) ?? null) !== null;
	} catch {
		return false;
	}
}

/** The message waiting for `chat`, taken out of the outbox; null when there's none. */
export function take(chat: string, storage = browserStorage()): string | null {
	try {
		const text = storage?.getItem(PREFIX + chat) ?? null;
		if (text !== null) storage?.removeItem(PREFIX + chat);
		return text;
	} catch {
		return null;
	}
}
