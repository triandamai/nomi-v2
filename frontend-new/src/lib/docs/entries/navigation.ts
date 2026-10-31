import type { ComponentDoc } from '../types';

export default [
	{
		file: 'navigation/NavGroupLabel.svelte',
		summary:
			'Names a group of sidebar rows (Talk, Build, Life) in small mono capitals. Anything placed beside it, like the workspace chip next to Build, sits at the end of the row.',
		use: ['Above each group of NavItems in the sidebar.', 'With a WorkspaceChip beside Build, to show whose projects are listed.'],
		avoid: [
			'A heading inside a card or panel: use SectionLabel.',
			'Groups inside a menu: use a { group } entry in Menu.',
		],
		dos: ['Write the label in sentence case; the capitals come from CSS.', 'Keep it to one word where you can.'],
		donts: ['Type the label in capitals.', 'Make it clickable: it names a group, it doesn\'t go anywhere.'],
		content: ['One word, sentence case in the code: "Talk", "Build", "Life".'],
		accessibility: ['A paragraph, not a heading. Screen readers read the words as written, not in capitals.'],
		related: ['navigation/NavItem.svelte', 'status/SectionLabel.svelte', 'chrome/WorkspaceChip.svelte'],
	},
	{
		file: 'navigation/NavItem.svelte',
		summary:
			'A sidebar row: an icon, a label and an optional count. The active row has a surface-2 fill and primary text. The container it sits in decides how far in it is and the tone of its icon.',
		use: ['Top-level places in the Sidebar: Home, Inbox, Chats, Projects, Money.', 'The pages in the Settings nav.'],
		avoid: [
			'A recent chat or project: use RecentRow.',
			'Switching panels within a page: use Tabs.',
			'An action rather than a place: use Button.',
		],
		dos: ['Mark the page people are on as active, and only that one.', 'Show a count when it tells people something: "Chats 12", "Inbox 3".'],
		donts: ['Two active rows at once.', 'Put toggles or buttons inside a nav row.'],
		content: ['One to three words, sentence case: "Chats", "Projects", "Plan & usage".', 'Counts are numbers or a short word in mono: "12", "Free".'],
		accessibility: [
			'A real link; the active row has aria-current="page".',
			'The icon is decorative; the label names the row.',
			'On touch screens the row is at least 44px tall.',
		],
		related: ['navigation/RecentRow.svelte', 'navigation/NavGroupLabel.svelte', 'chrome/Sidebar.svelte'],
	},
	{
		file: 'navigation/RecentRow.svelte',
		summary: 'A recent chat or project in the sidebar: a chat or folder icon, its title, and how long ago in mono ("2h", "2d").',
		use: ['The sidebar\'s Recents, through RecentsList.'],
		avoid: ['The main places in the app: use NavItem.', 'A project in a full list, with its status and URL: use ProjectRow.'],
		dos: ['Keep the time short and relative: "now", "2h", "2d".', 'Use the chat or project\'s own name.'],
		donts: ['Let a long title wrap: it is cut off on one line so the row stays 30px.'],
		content: ['Chat titles as they are ("Budget belanja Oktober"); project names exactly as deployed ("salah-daily-tracker").'],
		accessibility: [
			'A real link; the active row has aria-current="page".',
			'The icon is decorative. A cut-off title is only cut visually, so screen readers hear all of it.',
		],
		related: ['chrome/RecentsList.svelte', 'navigation/NavItem.svelte', 'lists/ProjectRow.svelte'],
	},
] satisfies ComponentDoc[];
