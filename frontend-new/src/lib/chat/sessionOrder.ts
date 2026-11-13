// Pinned chats sit on top of every list of chats, newest pin first; and the list's own search.
// pinnedFirst is ported from the old app's sessionOrder (frontend/src/lib/sessionOrder.ts).

type Pinnable = { pinned_at?: string | null; pinned?: boolean };

const isPinned = (chat: Pinnable) => Boolean(chat.pinned_at) || chat.pinned === true;

/** `chats` with the pinned ones first, newest pin first; the rest keep their order. */
export function pinnedFirst<T extends Pinnable>(chats: T[]): T[] {
	const pinned = chats.filter(isPinned).sort((a, b) => (b.pinned_at ?? '').localeCompare(a.pinned_at ?? ''));
	return [...pinned, ...chats.filter((chat) => !isPinned(chat))];
}

/** The chats holding every word of `query` in their title or last message, whatever the case. */
export function searchChats<T extends { title: string | null; last_message: { content: string } | null }>(chats: T[], query: string): T[] {
	const words = query.toLocaleLowerCase().split(/\s+/).filter(Boolean);
	if (words.length === 0) return chats;
	return chats.filter((chat) => {
		const text = `${chat.title ?? ''} ${chat.last_message?.content ?? ''}`.toLocaleLowerCase();
		return words.every((word) => text.includes(word));
	});
}
