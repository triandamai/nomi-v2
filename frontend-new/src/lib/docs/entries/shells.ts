import type { ComponentDoc } from '../types';

export default [
	{
		file: 'shells/AppShell.svelte',
		summary:
			'The frame for most screens (Home, Chats, Projects, Money…): sidebar 248, top bar 52, content padded 32–56, and an optional panel 300–380 on the right.',
		use: ['Any top-level screen people reach from the sidebar.'],
		avoid: ['Inside one project: use WorkspaceShell.', 'Onboarding, checkout or one focused task: use FocusShell.'],
		dos: [
			'Pick the shell before designing the screen.',
			'Keep the columns fixed and let the content fill.',
			'Put the crew panel in panel; it shows only where there is room.',
		],
		donts: ['Change the column widths for one screen.'],
		accessibility: [
			'The content is the page\'s <main>.',
			'Below 1024px the sidebar moves into a drawer, opened from the top bar\'s "Open navigation" button. The drawer is a native modal dialog and following a link closes it.',
			'The panel shows from 1280px up.',
		],
		related: ['shells/WorkspaceShell.svelte', 'shells/FocusShell.svelte', 'chrome/Sidebar.svelte'],
	},
	{
		file: 'shells/FocusShell.svelte',
		summary: 'One centred column for onboarding, dialogs as pages, and checkout: 480 wide (narrow) or 720 (wide), on the page background.',
		use: ['Onboarding, a first run, checkout: one task with nothing else around it.'],
		avoid: ['Screens people move around from: use AppShell.', 'Work inside a project: use WorkspaceShell.'],
		dos: ['Pick the shell before designing the screen.'],
		related: ['shells/AppShell.svelte', 'shells/WorkspaceShell.svelte', 'build/FirstRun.svelte'],
	},
	{
		file: 'shells/WorkspaceShell.svelte',
		summary:
			'The frame inside one project: a 52px project bar, the agent chat 440 wide on the left, then the preview, code or terminal. On phones Chat and Preview switch at the top instead of sitting side by side.',
		use: ['Everything inside one project.'],
		avoid: ['Screens outside a project: use AppShell.', 'One focused task: use FocusShell.'],
		dos: [
			'Pick the shell before designing the screen.',
			'Open project tools as a sheet over the preview, so the app people are looking at never closes.',
		],
		donts: ['Let the preview go below 480 or the chat outside 360–720 when the split is resizable.'],
		accessibility: [
			'The chat is a section labelled "Chat".',
			'On phones the view switch is a radio group labelled "View" (Chat, Preview); view is bindable.',
		],
		related: ['shells/AppShell.svelte', 'shells/FocusShell.svelte', 'build/ResizeHandle.svelte', 'build/WorkspaceSheet.svelte'],
	},
] satisfies ComponentDoc[];
