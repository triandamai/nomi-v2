// People in a shared project as the design system draws them (Flow / Build — Invite & build
// together TnwQT): initials, one of the two peer colours (the same wherever they show), where they
// are, and their pointers. Ported from the old app's peers.ts and projectPresence.svelte.ts.

export function initials(name: string): string {
	const words = name
		.trim()
		.split(/[\s._-]+/)
		.filter(Boolean);
	if (!words.length) return '?';
	return (words.length === 1 ? words[0].slice(0, 2) : words[0][0] + words[words.length - 1][0]).toUpperCase();
}

/**
 * Everyone's initials, none the same (the presence faces are told apart by them): a clash takes
 * the name's first two letters, then a number.
 */
export function distinctInitials(names: string[]): string[] {
	const taken = new Set<string>();
	return names.map((name) => {
		const first = initials(name);
		const letters = name.replace(/[^\p{L}\p{N}]/gu, '').slice(0, 2).toUpperCase() || first;
		let pick = taken.has(first) ? letters : first;
		for (let n = 2; taken.has(pick); n += 1) pick = `${letters}${n}`;
		taken.add(pick);
		return pick;
	});
}

/** Peer colour 1 or 2 for a person (by their name: chat messages carry no id), the same in the bar, the chat and the preview. */
export function peerFor(key: string): 1 | 2 {
	let hash = 0;
	for (const char of key.toLowerCase()) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
	return ((hash % 2) + 1) as 1 | 2;
}

/** Where someone is, in words: "Preview", "Code · src/app.ts", "Terminal". */
export function whereLabel(view: string | null | undefined, path: string | null | undefined, labels: { preview: string; code: string; terminal: string }): string {
	if (view === 'code') return path ? `${labels.code} · ${path}` : labels.code;
	if (view === 'terminal') return labels.terminal;
	return labels.preview;
}

/** Where someone's pointer is over a view, as fractions of it, and when it last moved (ms). */
export type PeerPointer = { name: string; x: number; y: number; view: string; at: number };

/** A pointer that hasn't moved for this long isn't shown. */
export const POINTER_TTL_MS = 5_000;

/** The pointers over one view that moved lately. */
export function pointersIn(pointers: Record<string, PeerPointer>, view: string, now: number): (PeerPointer & { user_id: string })[] {
	return Object.entries(pointers)
		.filter(([, pointer]) => pointer.view === view && now - pointer.at < POINTER_TTL_MS)
		.map(([user_id, pointer]) => ({ user_id, ...pointer }));
}
