import type { PatternDoc } from '../types';

// The Projects sheet, section by section: the page's parts and grid, then the card in every state
// with what floats over it. Drawn from the sheet's own notes (pen/export/projects--*.html) and the
// components that build it; quoted copy is the pen's.

export default [
	{
		sheet: 'projects',
		id: 'page-parts-grid',
		summary:
			'The Projects page on one grid: 384px columns with 16px gaps, two beside the detail panel and three when it’s closed. The top bar, the title and the composer sit above the toolbar, the project grid and the remix row. Nothing stretches to the window.',
		use: [
			'The Projects page itself, in any workspace.',
			'Starting a project from the page: the composer asks what to build and whether it needs a database.',
			'Offering apps other people built, to remix into your own copy.',
		],
		dos: [
			'Keep the header and the composer two columns wide (784), whether the panel is open or not.',
			'Let the toolbar, the grid and the remix row span the grid: 784 beside the panel, 1184 without it.',
			'Show the workspace in the top bar’s crumb, with its members and Invite beside it.',
			'Count each filter tab from the real projects: “All 4”, “Live 2”.',
		],
		donts: [
			'Don’t stretch cards or columns to fill a wide window; leave the space empty.',
			'Don’t centre or spread out a short row: cards sit left-aligned, with empty space after fewer than a full row.',
			'Don’t add a fourth column on a wider screen.',
		],
		content: [
			'Section heads say what they hold and link out: “Remix from the community”, “Browse showcase”.',
			'A remix keeps the maker’s data theirs: “You get your own copy — their data stays theirs.”',
			'The composer’s placeholder asks for the app in the person’s own words; the database choice says what it’s for: “for orders”.',
		],
	},
	{
		sheet: 'projects',
		id: 'cards-overlays',
		summary:
			'A project card is 384 × 192 and never changes size: who it is, a two-line status zone, then a footer with one action. The zone turns warning when Koda needs you and danger when something broke. Menus and dialogs act on one card; the panel shows the selected project.',
		use: [
			'Every project in the grid, in its current state: live, building, stopped, failed, draft, importing, waiting on answers, down, being renamed, or loading.',
			'Acting on a project without opening it: stop, resume, deploy, answer, fix, or move it to another workspace.',
			'Importing a repository from GitHub, and reviewing what Koda found before it deploys.',
			'Looking inside a project from the page: its code, address, data, deploys, members and chats.',
		],
		dos: [
			'Give the status zone exactly two lines: the status with its meta on the right (the step, the deploy or the time), then one detail line of up to about 45 characters.',
			'Pick the action by what the person must do: inverse when it needs them (Answer, Deploy, Ask Koda to fix), secondary to open or resume, ghost to stop or cancel.',
			'Say what’s safe before a destructive step: “nothing deployed · files kept”, “#5 still live”.',
			'Mark the selected card with the 2px stroke and open its panel beside the grid.',
			'Confirm a stop, a deploy and a move in a dialog that names the project and what will happen.',
		],
		donts: [
			'Don’t grow a card to fit a long detail; cut the line, the panel has the rest.',
			'Don’t show a percentage for a build; count the plan’s steps (“3 of 6”).',
			'Don’t tint a card in a crew colour; the zone is neutral, warning or danger, and only the app tile carries a tint.',
			'Don’t show more than one action in a card’s footer; the rest live in the ⋯ menu.',
		],
		content: [
			'Status lines name who and what: “Koda is building”, “Koda has 3 questions”, “Deploy failed”, “Down”.',
			'Details are machine facts in mono: “writing routes/menu/+page.svelte”, “build error in src/lib/pdf.ts · #5 still live”, “502 for visitors · last OK 09:10”.',
			'Footers count chats and say when: “2 chats · deploy #15 · 2d ago”, “from GitHub · started 09:02”.',
			'An import names the repository and says what Koda found and what it will set up, including the secrets it still needs.',
		],
	},
] satisfies PatternDoc[];
