import type { ComponentDoc } from '../types';

export default [
	{
		file: 'foundations/GithubMark.svelte',
		summary:
			'The GitHub glyph the pen draws on "Push to GitHub", shaped like a Lucide icon so it works anywhere an icon does. Lucide dropped its brand icons, so this one lives here.',
		use: ['As a button\'s icon where the action involves GitHub: "Push to GitHub".'],
		avoid: ['Any other icon: use Icon with a Lucide icon.'],
		dos: ['Pass it where an icon component is expected, such as Button\'s iconEnd or Icon\'s icon.'],
		accessibility: ['Hidden from screen readers; the button\'s words say what it does.'],
		related: ['foundations/Icon.svelte', 'build/ExportPanel.svelte'],
	},
	{
		file: 'foundations/Icon.svelte',
		summary:
			'Draws a Lucide icon at the design\'s 1.5 stroke and one of its sizes: 16 in nav and buttons, 14 inline with text, 12 in chips. Decorative unless it is given a label.',
		use: ['Every icon in the interface.'],
		avoid: ['A crew member: use Mascot.'],
		dos: [
			'Use Lucide icons only.',
			'Pick the size by where it sits: 16 in nav and buttons, 14 inline with text, 12 in chips; others only where the pen draws them.',
			'Give it a label when the icon means something on its own.',
		],
		donts: ['Emoji or illustrations in the interface.', 'A different stroke width or icon set.'],
		accessibility: ['Without a label it is decorative. With one it gets role="img" and that label.'],
		related: ['foundations/GithubMark.svelte', 'actions/IconButton.svelte'],
	},
	{
		file: 'foundations/VisuallyHidden.svelte',
		summary: 'Text only screen readers announce, such as what a count means or the state of an icon-only control.',
		use: ['Saying what a bare number means.', 'Adding a state that is only shown visually.'],
		avoid: ['Naming an icon-only button: IconButton takes a label of its own.'],
		dos: ['Keep it short and put it right next to what it explains.'],
		donts: ['Hide text sighted people also need.'],
		accessibility: ['Uses the sr-only class: out of sight, still read.'],
		related: ['foundations/Icon.svelte', 'actions/IconButton.svelte'],
	},
] satisfies ComponentDoc[];
