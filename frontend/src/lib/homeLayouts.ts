import { m } from '$lib/paraglide/messages';

// Home pages are configuration, not code. A layout is a backdrop plus rows of blocks; the home
// route renders whichever one the person picked in Preferences → Home (HomeLayout.svelte).
//
// To add a home page: add an entry to HOME_LAYOUTS (and its name + description to messages/).
// Rearranging or restyling blocks is a config change; only a brand-new kind of block needs a
// component (under $lib/components/home/blocks) and a case in HomeLayout.svelte. The backend
// stores just the slug, so it needs no change either.

/** One piece of the home page, with the options it takes. */
export type HomeBlock =
	| { block: 'greeting'; size: 'large' | 'hero' }
	| { block: 'composer'; suggestions: boolean }
	| { block: 'crew'; style: 'card' | 'strip'; limit: number }
	| { block: 'status' }
	| { block: 'recent'; style: 'cards' | 'list'; limit: number };
export type HomeBlockKind = HomeBlock['block'];

/** A row of the page: one column, or several side by side on wide screens. */
export interface HomeRow {
	columns: HomeBlock[][];
	/** Width of the row: the full page, or a narrow centred column. */
	width?: 'wide' | 'narrow';
	align?: 'start' | 'center';
	/** Wide-screen column widths (fr), one per column; equal when left out. */
	ratio?: number[];
}

export interface HomeLayout {
	id: string;
	name: () => string;
	description: () => string;
	/** What the page sits on: the plain ground, or the frosted crew stage (GlassBackdrop). */
	backdrop: 'plain' | 'glass';
	rows: HomeRow[];
}

export const HOME_LAYOUTS: readonly HomeLayout[] = [
	{
		id: 'classic',
		name: m.home_layout_classic,
		description: m.home_layout_classic_hint,
		backdrop: 'plain',
		rows: [
			{
				columns: [
					[{ block: 'greeting', size: 'large' }, { block: 'composer', suggestions: true }],
					[{ block: 'crew', style: 'card', limit: 4 }],
				],
				ratio: [1.5, 1],
			},
			{ columns: [[{ block: 'status' }]] },
			{ columns: [[{ block: 'recent', style: 'cards', limit: 6 }]] },
		],
	},
	{
		id: 'stage',
		name: m.home_layout_stage,
		description: m.home_layout_stage_hint,
		backdrop: 'glass',
		rows: [
			{
				width: 'narrow',
				align: 'center',
				columns: [[{ block: 'greeting', size: 'hero' }, { block: 'composer', suggestions: true }]],
			},
			{ width: 'narrow', columns: [[{ block: 'crew', style: 'strip', limit: 8 }]] },
			{ columns: [[{ block: 'status' }]] },
			{ width: 'narrow', columns: [[{ block: 'recent', style: 'list', limit: 5 }]] },
		],
	},
];

export const DEFAULT_HOME_LAYOUT = 'classic';

/** A slug of one of this build's layouts. */
export function isHomeLayoutId(value: unknown): value is string {
	return typeof value === 'string' && HOME_LAYOUTS.some((layout) => layout.id === value);
}

/** The layout for a stored slug; a slug this build doesn't know (or none) gets the default. */
export function homeLayout(id: string | null | undefined): HomeLayout {
	return HOME_LAYOUTS.find((layout) => layout.id === id) ?? HOME_LAYOUTS.find((layout) => layout.id === DEFAULT_HOME_LAYOUT)!;
}

/** Which blocks a layout uses, so the page only loads the data those blocks need. */
export function layoutBlocks(layout: HomeLayout): Set<HomeBlockKind> {
	return new Set(layout.rows.flatMap((row) => row.columns.flat().map((block) => block.block)));
}
