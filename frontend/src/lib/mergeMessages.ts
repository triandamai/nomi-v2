// When the chat's messages are reloaded from the server, live updates can already be ahead of
// that snapshot: a turn that fails in milliseconds posts its notice while the send is still
// reloading the page. Keep those instead of letting the older snapshot wipe them out.

interface Timed {
	id: string;
	created_at: string;
}

/** The reloaded `incoming` list, plus anything shown live that's newer than all of it. */
export function mergeReloaded<T extends Timed>(incoming: T[], shown: T[]): T[] {
	const known = new Set(incoming.map((m) => m.id));
	const newest = incoming.reduce((latest, m) => (m.created_at > latest ? m.created_at : latest), '');
	const ahead = shown.filter((m) => !known.has(m.id) && m.created_at > newest);
	return ahead.length ? [...incoming, ...ahead] : incoming;
}
