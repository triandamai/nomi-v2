import type { ComponentDoc } from '../types';

export default [
	{
		file: 'inputs/CheckMark.svelte',
		summary:
			'The 16px checkbox on its own, filled in text colour when checked. It is for rows that lay out their own label, such as a selection list or a many-of-many question; Checkbox wraps it with a plain text label.',
		use: ['Rows that bring their own name and detail, like the options in a selection card or a feedback form.'],
		avoid: [
			'A plain option with a one-line label: use Checkbox.',
			'A setting that takes effect at once: use Toggle.',
		],
		dos: ['Name it: wrap the box and its row text in a <label>, or pass aria-label.', 'Bind checked, or listen to onchange, to keep the row in step.'],
		donts: ['Leave it without a name. A box on its own says nothing to a screen reader.'],
		accessibility: [
			'A native checkbox input: Space ticks it, and it takes part in forms through name.',
			'Keyboard focus shows the focus ring 2px out from the box.',
			'The check glyph is decorative; the checked state comes from the input itself.',
		],
		related: ['inputs/Checkbox.svelte', 'inputs/Toggle.svelte', 'chat/SelectionCard.svelte'],
	},
	{
		file: 'inputs/Checkbox.svelte',
		summary: 'A checkbox with its text label: a 16px box, filled in text colour when checked. For options people tick in any combination before they go on.',
		use: ['Independent options that can be on together, such as what a shared view hides.'],
		avoid: [
			'A setting that applies the moment it changes: use Toggle.',
			'One choice out of a few: use Segmented.',
			'Rows with their own layout of name and detail: use CheckMark.',
		],
		dos: ['Word the label so that ticked means yes: "Mask amounts", "Include names".', 'Keep related options together, one per line.'],
		donts: ['Use a checkbox for a choice that has to be exactly one of several.', 'Write a question as the label.'],
		content: ['Sentence case, no full stop.', 'Say what ticking it does, not what it is about: "Include names", not "Names".'],
		accessibility: [
			'The label and the box are one <label>, so clicking the words ticks the box.',
			'A native checkbox: Space ticks it. Disabled dims it to 40% and takes it out of the tab order.',
		],
		related: ['inputs/CheckMark.svelte', 'inputs/Toggle.svelte', 'inputs/Segmented.svelte'],
	},
	{
		file: 'inputs/Chip.svelte',
		summary:
			'A small 30px pill. Model, option and agent chips show the current choice and open a menu to change it; provision chips name what a project was given (a git repo, a database) and aren\'t interactive.',
		use: [
			'model: the model in use, in mono ("deepseek-flash").',
			'agent: who a message goes to, with their mascot ("Nomi").',
			'option: a setting with a few values, like when a share link expires.',
			'provision: what a project has, such as "git repo", shown as a fact.',
		],
		avoid: [
			'A one-off action: use Button.',
			'Switching between views of a list: use Segmented.',
			'A verdict or a short marker like "Draft": use Tag.',
		],
		dos: ['Show the current value as the label.', 'Set expanded while its menu is open.', 'Open a Menu from it to change the value.'],
		donts: ['Make a provision chip look clickable or give it an action.', 'Put a prompt like "Choose model" in the label: show what is chosen.'],
		content: [
			'The label is the value itself: "deepseek-flash", "Nomi", "Default", "git repo".',
			'Model names exactly as the provider writes them, in mono.',
		],
		accessibility: [
			'Model, option and agent chips are buttons with aria-haspopup="menu" and aria-expanded.',
			'The chevron and the mascot are decorative; the label names the chip.',
			'On touch screens the target grows to 44px tall.',
			'A provision chip is plain text, not in the tab order.',
		],
		related: ['overlays/Menu.svelte', 'inputs/Composer.svelte', 'status/Tag.svelte'],
	},
	{
		file: 'inputs/Composer.svelte',
		summary:
			'Where people write to Nomi and the crew: the message field, attach, the agent and model chips, key hints, record and send. One component for chat, home and project; only the agent chip and the placeholder change.',
		use: ['The foot of a chat, the ask box on Home, and a project\'s chat.'],
		avoid: [
			'On phones: use the mobile Composer, which sits above the keyboard.',
			'Searching: use Search.',
			'A value in a form: use TextField.',
		],
		dos: [
			'Change the agent chip and placeholder to fit where it is; keep everything else the same.',
			'Write a placeholder that says who is listening: "Ask Nomi, or hand something to the crew…".',
			'Wire onpickagent and onpickmodel to a Menu so people can change either.',
		],
		donts: ['Build a second composer for a new screen.', 'Add a separate send button: send is built in and stays off until there is text.'],
		content: [
			'The placeholder names who reads the message and what they can do, in sentence case, ending with an ellipsis.',
			'The key hints stay as they are: "↵ send", "⇧↵ new line", "@ address crew".',
		],
		accessibility: [
			'The field is a textarea labelled "Message". Enter sends, Shift+Enter adds a line, and Enter never sends in the middle of an input-method composition.',
			'Attach, record and send are labelled icon buttons ("Attach a file", "Record a voice note", "Send").',
			'The field grows with the text up to 240px, then scrolls.',
			'The key hints are hidden on small screens.',
		],
		related: ['mobile/Composer.svelte', 'inputs/Chip.svelte', 'actions/KeyHint.svelte', 'actions/IconButton.svelte'],
	},
	{
		file: 'inputs/Search.svelte',
		summary:
			'A quiet 30px search box with a magnifier and, if there is one, its shortcut (⌘K). The plain version, used in the settings nav, sits 10px in and shows the shortcut as mono text instead of a key.',
		use: ['Searching chats, memory and files.', 'Filtering a list in place, like the settings or the projects in a picker.'],
		avoid: ['A value people enter and save: use TextField.', 'Writing to the crew: use Composer.'],
		dos: [
			'Say in the placeholder what it searches: "Search chats, memory, files".',
			'Show the shortcut when there is one.',
			'Make a narrower search say so: the move dialog\'s says "Search projects".',
		],
		donts: ['Reuse a generic placeholder where the search covers less.', 'Use it as a general text input.'],
		content: ['"Search" followed by what it searches, sentence case, no full stop.'],
		accessibility: [
			'A native type="search" input. Its name is label if you pass one, otherwise the placeholder.',
			'Focus anywhere in the box shows the accent border and the field glow.',
			'The shortcut is only shown; the page has to listen for the key.',
		],
		related: ['inputs/TextField.svelte', 'actions/Kbd.svelte', 'chrome/SettingsNav.svelte'],
	},
	{
		file: 'inputs/Segment.svelte',
		summary:
			'One option inside a Segmented control: active (on bg, outlined, medium weight) or not, with an optional count. It only draws the option; Segmented turns each one into a radio.',
		use: ['Inside Segmented, which places it for you.', 'On its own only to show one state as drawn, as the design sheets do.'],
		avoid: ['Placing it directly in a screen: use Segmented, which adds the radio group and the keyboard.'],
		donts: ['Use it as a button: it has no role and no keyboard handling of its own.'],
		related: ['inputs/Segmented.svelte', 'inputs/Tab.svelte'],
	},
	{
		file: 'inputs/Segmented.svelte',
		summary:
			'A few mutually exclusive views of one list, side by side, with optional counts ("All 4 · Live 2 · Building 1"). It works as a radio group: exactly one is chosen.',
		use: [
			'Switching between views of the same list.',
			'Choosing between two or three options inside a card, such as "This reply" or "Whole chat" when sharing.',
			'Switching Chat and Preview in the workspace on a phone.',
		],
		avoid: [
			'Separate panels of different content: use Tabs.',
			'A choice from a longer list: use a Chip with a Menu.',
			'On or off: use Toggle.',
		],
		dos: [
			'Keep to a few short options.',
			'Show counts when they help people choose.',
			'Inside a card use surface="bg" or "surface" (outlined), as the pen draws it.',
		],
		donts: ['Put actions in it: every option is a view or a value, never a command.', 'Use it when nothing is chosen at first: one option is always on.'],
		content: ['One or two words per option, sentence case: "All", "Live", "Building".', 'Counts are numbers, shown in mono.'],
		accessibility: [
			'role="radiogroup", named by label; each option is role="radio" with aria-checked.',
			'Only the chosen option is in the tab order. Arrow keys move the choice and wrap around, like the platform\'s own radios.',
			'On touch screens each option is at least 44px tall.',
		],
		related: ['inputs/Tabs.svelte', 'inputs/Segment.svelte', 'inputs/Chip.svelte'],
	},
	{
		file: 'inputs/Tab.svelte',
		summary: 'One tab label, 40px tall: the open one in text colour with a line under it. It only draws the tab; Tabs turns each one into a real tab.',
		use: ['Inside Tabs, which places it for you.', 'On its own only to show one state as drawn, as the design sheets do.'],
		avoid: ['Placing it directly in a screen: use Tabs, which adds the roles and the keyboard.'],
		donts: ['Use it as a button: it has no role and no keyboard handling of its own.'],
		related: ['inputs/Tabs.svelte', 'inputs/Segment.svelte'],
	},
	{
		file: 'inputs/Tabs.svelte',
		summary:
			'A row of tabs that switch between panels about one thing, such as a run\'s Transcript and Events. The open tab is in text colour with a line under it, and its panel shows below.',
		use: ['Different kinds of content about the same thing, one at a time.', 'Examples, Code and Usage on these docs pages.'],
		avoid: ['Filtering one list (All, Live, Building): use Segmented.', 'Moving to another page: use NavItem or Link.'],
		dos: ['Keep labels to one or two words.', 'Put the tab people need most first; it opens by default.'],
		donts: ['Hide content people need at the same time in different tabs.'],
		content: ['Short nouns in sentence case: "Transcript", "Events".'],
		accessibility: [
			'role="tablist", named by label; tabs have aria-selected and aria-controls; the panel is a tabpanel labelled by its tab.',
			'Left and Right move between tabs and open them, Home and End jump to the ends. Only the open tab is in the tab order.',
			'The panel can take focus, so keyboard users reach its content next.',
		],
		related: ['inputs/Segmented.svelte', 'inputs/Tab.svelte'],
	},
	{
		file: 'inputs/TextField.svelte',
		summary:
			'A labelled single-line field with an optional helper line. It sits on surface with border-strong; focus adds the focus ring; an error swaps the border and takes the helper\'s place to say what to do next.',
		use: ['Names, emails, URLs and keys.', 'Typing a project\'s name to confirm deleting it.'],
		avoid: ['Writing to the crew: use Composer.', 'Searching: use Search.', 'Choosing from set options: use Segmented, a Chip or Checkbox.'],
		dos: [
			'Use mono for machine values: slugs, keys, URLs.',
			'Say in the helper what is allowed and what it becomes: "Lowercase, numbers and dashes. Becomes your URL."',
			'Make the error say how to fix it, with an example: "Use lowercase and dashes, e.g. budget-keluarga."',
			'When it is disabled, say why: "Project name can\'t change after deploy."',
		],
		donts: ['An error that only says something is wrong ("Invalid name").', 'A placeholder instead of a label.'],
		content: [
			'The label is a short noun in sentence case: "Project name".',
			'Helper and error are full sentences with a full stop.',
		],
		accessibility: [
			'The label is tied to the input, and the helper or error is linked with aria-describedby.',
			'An error sets aria-invalid. Its icon is decorative; the words carry the message.',
			'type, name, required and autocomplete pass through to the input.',
		],
		related: ['inputs/Search.svelte', 'inputs/Composer.svelte'],
	},
	{
		file: 'inputs/Toggle.svelte',
		summary:
			'A switch for a setting that takes effect at once: mint with the knob on the right when on. Its label is for screen readers; the visible words sit beside it in the row.',
		use: ['Settings rows that apply immediately: "Hide amounts and names", "Anyone with the link can view".'],
		avoid: ['Options confirmed later with a button: use Checkbox.', 'Two or more named options: use Segmented.'],
		dos: [
			'Put the setting\'s name, and a line of help, beside it, and pass the same name as label.',
			'Apply the change as soon as it flips.',
		],
		donts: ['Use it to start an action: a toggle is a state, not a command.', 'Leave it without visible words next to it.'],
		content: [
			'Name the setting as it is when on: "Hide amounts and names".',
			'Add a help line that says what it means: "Shows Rp ••• and initials. Your records never leave Nomi."',
		],
		accessibility: [
			'A button with role="switch" and aria-checked; label becomes its aria-label.',
			'Space or Enter flips it. Disabled dims it to 40%.',
			'Keyboard focus shows the focus ring 2px out.',
		],
		related: ['inputs/Checkbox.svelte', 'inputs/Segmented.svelte', 'chat/SharePanel.svelte'],
	},
] satisfies ComponentDoc[];
