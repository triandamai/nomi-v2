// What the docs say about a component or a pattern, beyond what its code says. Written for
// everyone: designers, developers, content and marketing. Plain sentences, sentence case, the
// pen's own words where it has them (AGENTS.md §2 Copy).

export type ComponentDoc = {
	/** src/lib/ds path, e.g. actions/Button.svelte. */
	file: string;
	/** One or two sentences: what it is and what it's for. */
	summary: string;
	/** When to reach for it. */
	use?: string[];
	/** When not to, and what to use instead. */
	avoid?: string[];
	dos?: string[];
	donts?: string[];
	/** How to write its labels and text. */
	content?: string[];
	/** Keyboard, screen reader, focus and motion notes. */
	accessibility?: string[];
	/** Other components (src/lib/ds paths) people compare it with. */
	related?: string[];
};

export type PatternDoc = {
	sheet: 'chat' | 'build';
	/** The section's id on its sheet (slugify of its title). */
	id: string;
	summary: string;
	use?: string[];
	dos?: string[];
	donts?: string[];
	content?: string[];
};
