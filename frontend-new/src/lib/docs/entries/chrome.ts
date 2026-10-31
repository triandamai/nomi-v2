import type { ComponentDoc } from '../types';

export default [
	{
		file: 'chrome/RecentsList.svelte',
		summary: 'The sidebar\'s recent chats and projects under a foldable Recents header, always ending in a link to all chats.',
		use: ['In the Sidebar\'s recents slot.'],
		avoid: ['The main places in the app: use NavItem rows in the Sidebar.'],
		dos: ['Mix chats and projects in one list; the icon tells them apart.', 'Let people fold it; the "All chats" link stays either way.'],
		accessibility: [
			'The header is a button with aria-expanded and aria-controls; open is bindable.',
			'"All chats" stays when folded, so every chat is always one link away.',
		],
		related: ['navigation/RecentRow.svelte', 'chrome/Sidebar.svelte'],
	},
	{
		file: 'chrome/SettingsNav.svelte',
		summary:
			'The settings sidebar: back to Nomi (esc), a settings search, the groups of settings, and the person with Log out and the app\'s version at the foot.',
		use: ['Every settings page, in place of the Sidebar.'],
		avoid: ['The rest of the app: use Sidebar.'],
		dos: [
			'Group pages under short labels: Account, AI, Workspace, Data.',
			'Use counts for what is worth knowing at a glance: the plan ("Free"), the crew (10).',
		],
		accessibility: [
			'A <nav> labelled "Settings"; the active page has aria-current="page".',
			'The search is labelled "Search settings"; Log out is a real button.',
			'"esc" and "/" are only shown; the page has to listen for the keys.',
		],
		related: ['chrome/Sidebar.svelte', 'navigation/NavItem.svelte', 'inputs/Search.svelte'],
	},
	{
		file: 'chrome/Sidebar.svelte',
		summary:
			'The navigation on every app-shell screen: the brand and collapse, New chat (⌘K), the groups of nav rows, recents, then usage and the person at the foot. 248 wide.',
		use: ['The sidebar slot of AppShell.'],
		avoid: ['Settings pages: use SettingsNav.', 'Inside a project: the WorkspaceShell has its own project bar.'],
		dos: [
			'Fill it with NavItems under NavGroupLabels (Talk, Build, Life), RecentsList in recents, and UsageCard and UserRow in footer.',
			'Mark exactly one row active.',
		],
		accessibility: [
			'A <nav> labelled "Main".',
			'The collapse button ("Collapse sidebar") shows only when you pass oncollapse.',
			'New chat is a link that shows its shortcut; the page wires up the key.',
		],
		related: ['shells/AppShell.svelte', 'chrome/SettingsNav.svelte', 'navigation/NavItem.svelte', 'chrome/RecentsList.svelte'],
	},
	{
		file: 'chrome/UsageCard.svelte',
		summary: 'The plan and how much of it is used this month, at the foot of the sidebar, with a link to Plan & usage.',
		use: ['The Sidebar\'s footer, above UserRow.'],
		dos: ['Give the real figure: "21.8K / 1M tokens this month".'],
		accessibility: ['The meter is role="meter"; "Plan & usage" is a real link.'],
		related: ['status/Meter.svelte', 'chrome/Sidebar.svelte'],
	},
	{
		file: 'chrome/UserRow.svelte',
		summary: 'The person at the foot of the sidebar: their avatar, their name and the way into settings.',
		use: ['The Sidebar\'s footer, under UsageCard.'],
		accessibility: ['The settings link is an icon labelled "Settings"; on touch screens it is 44px.'],
		related: ['identity/Avatar.svelte', 'chrome/Sidebar.svelte'],
	},
	{
		file: 'chrome/WorkspaceChip.svelte',
		summary: 'Shows whose projects the Build group lists ("T Trian\'s space") and opens the workspace switcher. It sits beside the Build label in the sidebar.',
		use: ['Inside NavGroupLabel, beside Build.'],
		avoid: ['Picking a model or agent: use Chip.'],
		content: ['The workspace\'s name as people gave it: "Trian\'s space".'],
		accessibility: ['A button with aria-haspopup="menu"; the initial and the chevron are hidden from screen readers.'],
		related: ['navigation/NavGroupLabel.svelte', 'chrome/Sidebar.svelte', 'overlays/Menu.svelte'],
	},
] satisfies ComponentDoc[];
