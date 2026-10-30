// "@" in the composer: which crew member (or project file) the person is naming, and the text once
// they pick one. The server routes a message naming a crew member to them (nomi_agent_core::mention),
// so what goes in is the plain "@Name" anyone could type.
import type { MentionOption } from '$lib/components/ds/chat/mention';

/** How many options the picker lists at once. */
export const MAX_MENTION_OPTIONS = 8;

/** The "@query" being typed at `caret`: where its "@" is and what follows it. */
export function mentionAt(text: string, caret: number): { start: number; query: string } | null {
	const before = text.slice(0, caret);
	const match = /(^|[\s([{"'])@([\p{L}\p{N}_.\-/]*)$/u.exec(before);
	if (!match) return null;
	return { start: caret - match[2].length - 1, query: match[2] };
}

/** The options whose name (or, for a file, its path) starts with `query`, then those that contain it. */
export function filterMentions(options: MentionOption[], query: string): MentionOption[] {
	const q = query.toLowerCase();
	const name = (option: MentionOption) => option.name.toLowerCase();
	const starts = options.filter((option) => name(option).startsWith(q));
	const contains = options.filter((option) => !name(option).startsWith(q) && name(option).includes(q));
	return [...starts, ...contains].slice(0, MAX_MENTION_OPTIONS);
}

/** `text` with the "@query" from `start` to `caret` replaced by the picked name and a space. */
export function insertMention(text: string, start: number, caret: number, name: string): { text: string; caret: number } {
	const token = `@${name} `;
	const after = text.slice(caret).replace(/^ /, '');
	return { text: text.slice(0, start) + token + after, caret: start + token.length };
}
