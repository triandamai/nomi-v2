import type { ComponentDoc } from '../types';

export default [
	{
		file: 'overlays/Dialog.svelte',
		summary:
			'A modal dialog on the overlay scrim, for a decision that needs full attention, such as deleting a project. 420px wide (440 for a picker), with the title and description over the content and the actions in a footer.',
		use: ['Confirming something that can\'t be undone.', 'A short task that has to finish before going on, like moving a chat to a project.'],
		avoid: [
			'A panel that brings its own box (Share, Export): use Modal.',
			'A quick choice off a button: use Menu.',
			'Telling people something just happened: use Toast.',
		],
		dos: [
			'Ask the question with the real name in the title: "Delete salah-daily-tracker?".',
			'Say what goes, what stays and how to keep a copy: "Export repo first".',
			'Put Cancel (secondary) before the action, and make a destructive action danger.',
		],
		donts: ['A mint primary in a dialog: the main action is inverse, or danger when it destroys.', 'A dialog for news that needs no decision.'],
		content: [
			'The title is a question in sentence case, naming the thing.',
			'The action repeats the verb from the title: "Delete project", not "OK".',
		],
		accessibility: [
			'A native <dialog> opened with showModal: it sits in the top layer, keeps focus inside, and Escape closes it.',
			'Named by its title (aria-labelledby) and described by its description.',
			'open is bindable, and onclose runs whenever it closes.',
		],
		related: ['overlays/DialogPanel.svelte', 'overlays/Modal.svelte', 'overlays/Menu.svelte'],
	},
	{
		file: 'overlays/DialogPanel.svelte',
		summary:
			'The dialog\'s box on its own: surface, a border-strong edge, the one shadow, the title and description over the content, and the actions in a footer. Dialog puts it in a native modal; the sheets show it in place.',
		use: ['Through Dialog.', 'On its own only to draw a dialog open, as the design sheets do.'],
		avoid: ['Placing it in a screen yourself: use Dialog, which adds the modal, the scrim and the focus handling.'],
		accessibility: ['Its title and description take ids from the id you pass ({id}-title, {id}-description), which Dialog points to. On its own it has no dialog role.'],
		related: ['overlays/Dialog.svelte', 'overlays/Modal.svelte'],
	},
	{
		file: 'overlays/Menu.svelte',
		summary:
			'A menu off a button: a floating panel of actions with icons, shortcuts and, last, a danger item. It sits in the top layer so nothing clips it, and the keyboard works as in the platform\'s own menus.',
		use: ['Actions on a project or a chat, off a "···" or a named button.', 'Changing the model or the agent from a Chip.'],
		avoid: [
			'One or two actions: show them as Buttons.',
			'Switching views: use Segmented.',
			'A decision that needs full attention: use Dialog.',
		],
		dos: [
			'Put the destructive item last, after a divider, as danger.',
			'Show the shortcut for items that have one.',
			'Use a { group } entry to name groups, and footer for a quiet note.',
		],
		donts: ['Mix the destructive item in with the others.'],
		content: [
			'Items are a verb and a thing, sentence case: "Export repo", "Visit live site", "Delete project".',
			'A title names what the menu acts on: "salah-daily-tracker".',
		],
		accessibility: [
			'Spread the props the trigger snippet gives you onto your button: aria-haspopup="menu", aria-expanded and aria-controls.',
			'Opening it focuses the first item. Arrow keys, Home and End move; Enter or Space picks; Escape closes and returns focus to the button; Tab or a click elsewhere closes it.',
			'Items are role="menuitem"; a named group is role="group".',
		],
		related: ['overlays/MenuPanel.svelte', 'overlays/MenuRow.svelte', 'inputs/Chip.svelte', 'overlays/Dialog.svelte'],
	},
	{
		file: 'overlays/MenuPanel.svelte',
		summary:
			'The menu\'s panel on its own: surface, a border-strong edge, the one shadow, rows 30px tall, with an optional title, named groups and a footer. Menu floats it off a button; the sheets show it open.',
		use: ['Through Menu.', 'On its own only to draw a menu open, as the design sheets do.'],
		avoid: ['Placing it in a screen yourself: use Menu, which adds positioning and the keyboard.'],
		dos: ['Use wide for menus whose rows need more room (280), or width 260 for the chat menu.'],
		accessibility: ['role="menu". Shown on its own, give it an aria-label.'],
		related: ['overlays/Menu.svelte', 'overlays/MenuRow.svelte'],
	},
	{
		file: 'overlays/MenuRow.svelte',
		summary: 'One menu item, 30px tall: an icon, a label, and a shortcut or meta in mono. Danger items are red. Wide rows are 32px with more room between their parts.',
		use: ['Inside MenuPanel; Menu builds the rows from its items.'],
		avoid: ['A place to go in the sidebar: use NavItem.'],
		dos: ['Use danger only for destructive actions.'],
		accessibility: [
			'role="menuitem" with tabindex -1: Menu moves focus between rows. It must sit inside a role="menu".',
			'Focus shows the same fill as hover. On touch screens rows are 44px tall.',
		],
		related: ['overlays/MenuPanel.svelte', 'overlays/Menu.svelte'],
	},
	{
		file: 'overlays/Modal.svelte',
		summary:
			'A native modal for a panel that brings its own box, such as Share or Export: the overlay scrim, the top layer, focus kept inside, and Escape to close. It is named by the panel\'s own heading.',
		use: ['Panels with their own layout and close button: Share, Export.'],
		avoid: ['A standard confirmation: use Dialog, which has the title, description and actions built in.'],
		dos: ['Pass labelledby: the id of the panel\'s heading.', 'Give the panel its own way to close.'],
		accessibility: [
			'A native <dialog> opened with showModal; Escape closes it.',
			'open is bindable, and onclose runs whenever it closes.',
		],
		related: ['overlays/Dialog.svelte', 'build/ExportDialog.svelte', 'build/BuildShareDialog.svelte'],
	},
	{
		file: 'overlays/Toast.svelte',
		summary:
			'A short note about what just happened, with its detail in mono, one action and a close button. Success, error or info. Errors interrupt; everything else waits its turn.',
		use: ['A deploy finished, a repo was exported, a deploy failed.'],
		avoid: [
			'A failure tied to one message in a chat: use ErrorCallout.',
			'Something about what\'s around the chat that stays true for a while: use Banner.',
			'A question that needs an answer: use Dialog.',
		],
		dos: [
			'Name the real thing in the title: "Deployed to salah-daily-tracker-0ab8f8.trian.space".',
			'Put the numbers in the detail: "deploy #15 · built in 38s".',
			'Offer at most one action: "Visit", "Open log", "Show".',
			'Make an error say the next move: "Koda can try a fix, or open the log."',
		],
		donts: ['Two actions.', 'An error toast that only says it failed.'],
		content: [
			'The title says what happened, with the name: "Repo exported: salah-daily-tracker.zip".',
			'The detail is short and factual, in mono: "23 files · 184 KB".',
		],
		accessibility: [
			'Errors are role="alert" and are announced at once; success and info are role="status".',
			'The close button is labelled "Dismiss".',
		],
		related: ['overlays/ToastRegion.svelte', 'conversation/ErrorCallout.svelte', 'chat/Banner.svelte', 'mobile/PhoneToast.svelte'],
	},
	{
		file: 'overlays/ToastRegion.svelte',
		summary:
			'Where the app\'s toasts appear: a stack at the bottom right (full width on small screens). Show one from anywhere with toasts.show(); errors stay until dismissed, the rest go after 6 seconds.',
		use: ['Once, in the app\'s layout.'],
		avoid: ['Placing a single Toast yourself in a screen.'],
		dos: [
			'Render it once, near the root.',
			'Give a toast a longer duration when people need time to act on it, such as 10 seconds for an Undo.',
		],
		donts: ['Render more than one region.'],
		accessibility: [
			'A section labelled "Notifications".',
			'Pressing a toast\'s action also dismisses it; every toast can be dismissed.',
		],
		related: ['overlays/Toast.svelte'],
	},
	{
		file: 'overlays/Tooltip.svelte',
		summary:
			'A small inverted tip that names its trigger, with the shortcut if there is one. It shows on hover after a moment, or on keyboard focus; Escape hides it.',
		use: ['Icon buttons, so pointer users see their name.', 'Controls with a shortcut.'],
		avoid: ['Anything people must read to carry on: put it on the page, such as a TextField helper.'],
		dos: ['Spread the props it gives you onto the trigger (aria-describedby).', 'Use the same words as the trigger\'s label.'],
		donts: ['A tooltip on something that already shows the same words.'],
		content: ['A name, sentence case, no full stop: "New chat". The shortcut goes in shortcut: "⌘K".'],
		accessibility: [
			'role="tooltip", and it describes its trigger through aria-describedby.',
			'It shows 300ms after the pointer arrives, at once on keyboard focus, and Escape hides it. It never takes focus.',
		],
		related: ['overlays/TooltipBubble.svelte', 'actions/IconButton.svelte', 'actions/Kbd.svelte'],
	},
	{
		file: 'overlays/TooltipBubble.svelte',
		summary: 'The tooltip\'s inverted tip on its own, with the shortcut in mono. Tooltip shows it on hover or focus; the sheets draw it in place.',
		use: ['Through Tooltip.', 'On its own only to draw a tooltip open, as the design sheets do.'],
		avoid: ['Placing it in a screen yourself: use Tooltip, which positions it and ties it to its trigger.'],
		accessibility: ['role="tooltip"; it ignores the pointer.'],
		related: ['overlays/Tooltip.svelte'],
	},
] satisfies ComponentDoc[];
