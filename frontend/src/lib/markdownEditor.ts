// The blocks the description editor's "/" menu offers, and how it finds what's being typed after
// the slash. The editor itself is MarkdownEditor.svelte.

export type SlashBlock = 'text' | 'h1' | 'h2' | 'h3' | 'bullets' | 'numbers' | 'todo' | 'quote' | 'code' | 'divider';

/** Every block, in the order the menu lists them. */
export const SLASH_BLOCKS: SlashBlock[] = ['text', 'h1', 'h2', 'h3', 'bullets', 'numbers', 'todo', 'quote', 'code', 'divider'];

/** The longest word after "/" that still counts as looking for a block. */
export const MAX_SLASH_QUERY = 20;

/**
 * What's typed after a "/" that starts a word at the end of `before` (the text before the
 * cursor in its block), or null when the cursor isn't after one.
 */
export function slashQuery(before: string): string | null {
	const match = /(?:^|\s)\/([\p{L}\p{N}]*)$/u.exec(before);
	if (!match || match[1].length > MAX_SLASH_QUERY) return null;
	return match[1];
}

/** The blocks whose label (or any of its other names) starts a word with `query`. */
export function slashResults(query: string, names: (block: SlashBlock) => string[]): SlashBlock[] {
	const q = query.trim().toLowerCase();
	if (!q) return SLASH_BLOCKS;
	return SLASH_BLOCKS.filter((block) =>
		names(block).some((name) =>
			name
				.toLowerCase()
				.split(/[\s-]+/)
				.some((word) => word.startsWith(q)),
		),
	);
}
