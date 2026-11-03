// People in a shared project as the design system draws them (Build together): initials, and one
// of the two peer colours, in the order they joined. You stay neutral.

export function initials(name: string): string {
	const words = name
		.trim()
		.split(/[\s._-]+/)
		.filter(Boolean);
	if (words.length === 0) return '?';
	const letters = words.length === 1 ? words[0].slice(0, 2) : words[0][0] + words[words.length - 1][0];
	return letters.toUpperCase();
}

/** Peer colour 1 or 2, alternating by position. */
export const peerOf = (index: number): 1 | 2 => ((index % 2) + 1) as 1 | 2;

/** Peer colour 1 or 2 for a person, the same wherever they show (presence, chat, typing). */
export function peerFor(key: string): 1 | 2 {
	let hash = 0;
	for (const char of key.toLowerCase()) hash = (hash * 31 + char.charCodeAt(0)) >>> 0;
	return peerOf(hash);
}

/** Where someone is, in words: "Preview", "Code · src/app.ts", "Terminal". */
export function whereLabel(
	view: string | null | undefined,
	path: string | null | undefined,
	labels: { preview: string; code: string; terminal: string },
): string {
	if (view === 'code') return path ? `${labels.code} · ${path}` : labels.code;
	if (view === 'terminal') return labels.terminal;
	return labels.preview;
}
