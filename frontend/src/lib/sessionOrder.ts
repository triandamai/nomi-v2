// Pinned chats sit on top of every list of chats (the drawer's Recents, Chats), newest pin first.

/** `chats` with the pinned ones first, newest pin first; the rest keep their order. */
export function pinnedFirst<T extends { pinned_at: string | null }>(chats: T[]): T[] {
	const pinned = chats.filter((chat) => chat.pinned_at !== null).sort((a, b) => b.pinned_at!.localeCompare(a.pinned_at!));
	return [...pinned, ...chats.filter((chat) => chat.pinned_at === null)];
}
