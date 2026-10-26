// Who's typing in a project's chat and which live updates are about publishing. Typing pings
// come over the project's live topic every few seconds while someone writes; a name stays up
// until its last ping is older than the time to live.

export interface TypingEntry {
	name: string;
	/** When their first ping of this stretch of typing arrived (ms): the order names show in. */
	since: number;
	/** When their last ping arrived (ms): what the time to live counts from. */
	at: number;
}

export type TypingMap = Record<string, TypingEntry>;

/** Everyone typing except `me`, the one who started first first. */
export function typingNames(typing: TypingMap, me: string): string[] {
	return Object.entries(typing)
		.filter(([userId]) => userId !== me)
		.sort(([, a], [, b]) => a.since - b.since)
		.map(([, entry]) => entry.name);
}

/** `typing` with a ping from `userId` at `now`: a new name starts now, a known one stays in its place. */
export function withPing(typing: TypingMap, userId: string, name: string, now: number): TypingMap {
	return { ...typing, [userId]: { name, since: typing[userId]?.since ?? now, at: now } };
}

/** `typing` without the people whose last ping is older than `ttlMs`. */
export function expireTyping(typing: TypingMap, now: number, ttlMs: number): TypingMap {
	return Object.fromEntries(Object.entries(typing).filter(([, entry]) => now - entry.at < ttlMs));
}

/** Whether a ping may go out now, sending at most one every `everyMs`. */
export function shouldSend(lastSentAt: number | null, now: number, everyMs: number): boolean {
	return lastSentAt === null || now - lastSentAt >= everyMs;
}

/** Whether a message on the project's live topic is about publishing (so the Deploy state reloads). */
export function isDeployEvent(data: string): boolean {
	try {
		const kind = (JSON.parse(data) as { kind?: unknown }).kind;
		return kind !== 'PresenceChanged' && kind !== 'Typing';
	} catch {
		return true;
	}
}
