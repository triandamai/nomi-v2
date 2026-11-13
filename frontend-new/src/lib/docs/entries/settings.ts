import type { ComponentDoc } from '../types';

// The parts of a settings page (App / Settings / General m7lvY1): groups of rows, and the page's
// own column (On this page, Related, the tip).

export default [
	{
		file: 'settings/SettingsGroup.svelte',
		summary: 'One group of settings: its name and what it covers, over its rows.',
		use: ['Each section of a settings page that saves as it goes (General).'],
		avoid: ['A form saved as a whole: lay it out with its fields and a SaveBar.'],
		content: ['Name it in sentence case: "Account & sign-in".'],
		accessibility: ['A section named by its heading; its id is the anchor On this page links to.'],
		related: ['settings/SettingRow.svelte', 'settings/PageToc.svelte'],
	},
	{
		file: 'settings/SettingRow.svelte',
		summary: 'One setting: its name, what it means, and its control at the end.',
		use: ['Every row of a SettingsGroup: a switch, a segmented choice, a select or a button.'],
		dos: ['Write a machine’s value in mono (an email, a format).', 'Save as soon as the control changes, and put it back if saving fails.'],
		donts: ['Draw a control the backend can’t keep: leave the row out.'],
		accessibility: ['The control is named for the setting.'],
		related: ['settings/SettingsGroup.svelte', 'inbox/PreferenceRow.svelte'],
	},
	{
		file: 'settings/SettingNote.svelte',
		summary: 'A quiet note under a group’s rows, with the info glyph.',
		use: ['What a group doesn’t do, said plainly ("Signing in with Google doesn’t give the crew your Gmail").'],
		related: ['settings/SettingsGroup.svelte'],
	},
	{
		file: 'settings/PageToc.svelte',
		summary: 'A long settings page’s groups as links down a rule, the one in view marked.',
		use: ['The column beside General.'],
		accessibility: ['A navigation landmark; the group in view is aria-current="location".'],
		related: ['settings/SettingsGroup.svelte', 'settings/RelatedLinks.svelte'],
	},
	{
		file: 'settings/RelatedLinks.svelte',
		summary: 'Other places that touch these settings, each with its glyph and an arrow.',
		use: ['Under On this page.'],
		donts: ['Link to a page that doesn’t exist yet.'],
		related: ['settings/PageToc.svelte'],
	},
	{
		file: 'settings/SettingsTip.svelte',
		summary: 'One line about how the page works, under a stroke, with the accent check.',
		use: ['The foot of a settings page’s column.'],
		related: ['settings/AutosaveNote.svelte'],
	},
	{
		file: 'settings/AutosaveNote.svelte',
		summary: 'Says in the top bar that the page saves each change as it’s made.',
		use: ['The top bar of a settings page with no Save button.'],
		avoid: ['A page saved as a whole: it has a SaveBar instead.'],
		related: ['chrome/SettingsTopBar.svelte', 'settings/SettingsTip.svelte'],
	},
] satisfies ComponentDoc[];
