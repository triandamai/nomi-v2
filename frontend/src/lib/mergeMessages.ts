// When the chat's messages are reloaded from the server, live updates can already be ahead of
// that snapshot: a turn that fails in milliseconds posts its notice while the send is still
// reloading the page. Keep those instead of letting the older snapshot wipe them out.

interface Timed {
	id: string;
	created_at: string;
}

/**
 * The reloaded `incoming` list (the newest page), plus anything shown live that's newer than all
 * of it. When `incoming` is only the newest page of a longer chat (`partial`), the earlier pages
 * already loaded ("load earlier") stay too; otherwise anything older is gone (deleted).
 */
export function mergeReloaded<T extends Timed>(incoming: T[], shown: T[], partial = false): T[] {
	const known = new Set(incoming.map((m) => m.id));
	const newest = incoming.reduce((latest, m) => (m.created_at > latest ? m.created_at : latest), '');
	const oldest = incoming.reduce((first, m) => (first === '' || m.created_at < first ? m.created_at : first), '');
	const ahead = shown.filter((m) => !known.has(m.id) && m.created_at > newest);
	const behind = partial && oldest ? shown.filter((m) => !known.has(m.id) && m.created_at < oldest) : [];
	return ahead.length || behind.length ? [...behind, ...incoming, ...ahead] : incoming;
}

/** `older` (an earlier page) in front of `shown`, without repeating anything already shown. */
export function prependOlder<T extends Timed>(older: T[], shown: T[]): T[] {
	const known = new Set(shown.map((m) => m.id));
	return [...older.filter((m) => !known.has(m.id)), ...shown];
}
