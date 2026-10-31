import type { ComponentDoc } from '../types';

export default [
	{
		file: 'actions/Button.svelte',
		summary:
			'The button for doing something: starting a build, adding credits, deleting a project. It comes in six variants, all 30px tall. Primary (mint) marks the one thing to act on in a view; inverse is the main action inside a panel.',
		use: [
			'To start, confirm or cancel something: "Start building", "Delete project", "Cancel".',
			'As a link that looks like an action: pass href, and external when it goes to another site.',
			'While the action runs: loading, with real progress in the label ("Deploying · 00:14").',
		],
		avoid: [
			'Controls shown only as an icon: use IconButton.',
			'Going somewhere from a sentence or a card footer: use Link.',
			'Picking a model, an option or a crew member from a menu: use Chip.',
		],
		dos: [
			'Use one primary per view, and inverse for the main action inside a panel, so mint stays rare.',
			'Use danger for something that destroys data ("Delete project"), and ghost-danger for a quiet stop ("Stop all").',
			'Pair the main action with a secondary "Cancel" in a dialog footer.',
			'While loading, name the step and the time so far: "Deploying · 00:14".',
		],
		donts: [
			'Two mint primaries on one screen. If two things are mint, one of them is wrong.',
			'A primary inside a panel or dialog: the panel\'s main action is inverse.',
			'A spinner with no words, or a label that pretends to know how long is left.',
			'Title Case or ALL CAPS labels.',
		],
		content: [
			'Start with a verb and name the thing: "Open workspace", "Add credits on DeepSeek", "Delete project".',
			'Sentence case, no full stop.',
			'A loading label says what is happening and for how long, in the same words the screen uses: "Deploying · 00:14".',
		],
		accessibility: [
			'A real <button> (type="button" unless you say otherwise), or an <a> when it has an href. External links open in a new tab with rel="noopener noreferrer".',
			'Loading disables it and sets aria-busy; the spinner stops when people ask for reduced motion.',
			'Keyboard focus shows a 3px focus ring. On touch screens it is at least 44px tall.',
			'aria-* attributes pass through, for example aria-expanded on a button that opens something.',
		],
		related: ['actions/IconButton.svelte', 'actions/Link.svelte', 'inputs/Chip.svelte'],
	},
	{
		file: 'actions/IconButton.svelte',
		summary:
			'A 30×30 button with a 16px icon and no visible text, for compact tools such as attach, record and send. Its label is what screen readers say and what its tooltip shows, so it is never unnamed.',
		use: [
			'Tools people recognise by their icon: add, attach a file, record a voice note, send.',
			'Card headers and tight rows where a word would crowd the line (iconSize 14 where the pen draws a smaller glyph).',
			'The composer\'s send: the send variant, mint once there is something to send.',
		],
		avoid: [
			'An action that needs words to be understood: use Button, with iconStart if the icon helps.',
			'The main action of a view: use Button (primary).',
		],
		dos: [
			'Give every icon button a label that says what it does: "Record a voice note", "Send".',
			'Wrap it in Tooltip so pointer users see the same label, and the shortcut if there is one.',
			'Keep send disabled until there is something to send.',
		],
		donts: [
			'An icon that only makes sense with a word beside it.',
			'The send variant for anything other than sending.',
		],
		content: ['The label is a short verb phrase in sentence case: "Attach a file", "Record a voice note", "Send".'],
		accessibility: [
			'The label becomes the button\'s aria-label; the icon itself is decorative.',
			'Keyboard focus shows the focus ring. On touch screens the target grows to 44×44.',
			'Disabled uses the native disabled attribute, so it is skipped by Tab.',
		],
		related: ['actions/Button.svelte', 'overlays/Tooltip.svelte', 'inputs/Composer.svelte'],
	},
	{
		file: 'actions/Kbd.svelte',
		summary: 'A keyboard key or chord in a small mono box, such as ⌘K. Use it where the interface names a shortcut people can press.',
		use: ['Next to the control a shortcut triggers, like the search field\'s ⌘K.', 'In help text that names a key.'],
		avoid: [
			'Several keys with what each one does, under a field: use KeyHint.',
			'Shortcuts in menu rows and tooltips: MenuRow and TooltipBubble show them as plain mono text through their shortcut prop.',
		],
		dos: ['Write keys the way the platform shows them: ⌘K, ⇧↵, esc.', 'Keep it beside the thing it triggers.'],
		donts: ['Use it for anything that isn\'t a key, such as a count or a tag: use Tag or a mono count.'],
		content: [
			'Use the symbols ⌘ ⇧ ↵. Glyphs Geist Mono lacks fall back to the system font, as in the pen.',
			'One key or one chord per Kbd.',
		],
		accessibility: [
			'A native <kbd>, so it is marked up as keyboard input.',
			'It only describes a shortcut. The shortcut itself has to be wired up by the page.',
		],
		related: ['actions/KeyHint.svelte', 'inputs/Search.svelte', 'overlays/TooltipBubble.svelte'],
	},
	{
		file: 'actions/KeyHint.svelte',
		summary: 'A quiet mono line that says what the keys do here, such as "↵ send  ⇧↵ new line  @ address crew". It sits with the field it describes.',
		use: ['Under or inside a field whose keys do something people might not expect, like the composer.'],
		avoid: ['A single shortcut next to its control: use Kbd.'],
		dos: [
			'List only the few keys that matter in this field; the composer shows three.',
			'Drop it where there is no room. The composer shows it from medium screens up.',
		],
		donts: ['A full list of shortcuts.', 'Instructions in sentences: use a TextField helper instead.'],
		content: ['The key, then what it does in lower case: "↵ send", "⇧↵ new line", "@ address crew".', 'Short verbs, no full stop.'],
		accessibility: ['Plain text in a paragraph, not interactive.', 'Every key it lists must really work in the field it sits with.'],
		related: ['actions/Kbd.svelte', 'inputs/Composer.svelte'],
	},
	{
		file: 'actions/Link.svelte',
		summary:
			'A link to another page or site. Standalone, it stands on its own line with an arrow ("Upgrade to Pro ↗"); inline, it sits inside a sentence in text-2 ("…or switch to Nomi credits"). Both underline on hover.',
		use: [
			'Going somewhere rather than doing something: a settings page, a provider\'s site.',
			'Standalone under a card or an empty state: "Add reminder", "Upgrade to Pro".',
			'Inline, to carry on a sentence.',
			'Quiet (standalone, secondary text) for a link in a card\'s footer, such as "Open in Sheets".',
		],
		avoid: [
			'Starting or confirming an action: use Button.',
			'A link inside an agent\'s answer: use Hyperlink, which takes the sentence\'s size.',
		],
		dos: ['Mark links to another site external, so they open in a new tab.', 'Make the words say where the link goes.'],
		donts: ['"Click here" or a bare "Learn more".', 'Mint or a crew colour on a link: links stay neutral.'],
		content: [
			'Name the destination or what people will do there: "Upgrade to Pro", "switch to Nomi credits".',
			'An inline link continues the sentence, in lower case unless it starts it.',
		],
		accessibility: [
			'A real <a>. External links get target="_blank" and rel="noopener noreferrer".',
			'The arrow on a standalone link is decorative.',
			'Keyboard focus shows the focus ring around the link.',
		],
		related: ['actions/Button.svelte', 'chat/Hyperlink.svelte', 'lists/EmptyState.svelte'],
	},
] satisfies ComponentDoc[];
