// Which features are pinned to the navigation drawer, and in what order (see $lib/features).

export const PINNABLE = ['money', 'reminders', 'memory', 'notifications', 'crew', 'connections', 'models', 'billing'] as const;
export type PinnableId = (typeof PINNABLE)[number];

/** What the drawer showed before it could be customised. */
export const DEFAULT_PINS: PinnableId[] = ['money', 'reminders', 'memory', 'notifications'];

export function isPinnable(id: string): id is PinnableId {
	return (PINNABLE as readonly string[]).includes(id);
}

/** The saved pins (null until customised), cleaned of anything unknown or repeated. */
export function resolvePins(saved: string[] | null | undefined): PinnableId[] {
	if (!saved) return [...DEFAULT_PINS];
	const pins: PinnableId[] = [];
	for (const id of saved) if (isPinnable(id) && !pins.includes(id)) pins.push(id);
	return pins;
}

/** `list` with the item at `from` moved to `to`. */
export function moveItem<T>(list: T[], from: number, to: number): T[] {
	if (from === to || from < 0 || from >= list.length) return list;
	const next = [...list];
	const [item] = next.splice(from, 1);
	next.splice(Math.max(0, Math.min(to, next.length)), 0, item);
	return next;
}
