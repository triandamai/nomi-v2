import type { ComponentDoc } from '../types';

export default [
	{
		file: 'status/Meter.svelte',
		summary:
			'How much of something is used: a thin bar with the percentage, and the real figure in words under it ("21.8K / 1M tokens this month"). It never shows more than full.',
		use: ['Plan usage at the foot of the sidebar, through UsageCard.', 'Any quota with a known limit.'],
		avoid: ['Work whose length nobody knows: show the step and the time so far with LiveStep.'],
		dos: ['Give the real figure with its unit in the caption.', 'Use roomy inside the sidebar\'s usage card, as the pen does.'],
		donts: ['A bar that creeps toward 90% to look busy.', 'Colour the fill: it stays neutral.'],
		content: ['The label is what is measured: "Free plan".', 'The caption is the real figure: "21.8K / 1M tokens this month".'],
		accessibility: [
			'role="meter" from 0 to 100, named by its label, with aria-valuetext that includes the caption.',
			'Values below 0 or above 1 are held at empty or full.',
		],
		related: ['chrome/UsageCard.svelte', 'conversation/LiveStep.svelte'],
	},
	{
		file: 'status/SectionLabel.svelte',
		summary: 'A section\'s name in small mono capitals, with how many are in it ("RECENTS 4"). It is the only uppercase style in Nomi.',
		use: ['Naming a small group inside a card or panel: "Sources", "Follow up", "Included", "Send to".'],
		avoid: ['Groups in the sidebar: use NavGroupLabel.', 'Page and card titles: use a heading in sentence case.'],
		dos: ['Write it in sentence case; the capitals come from CSS.', 'Add a count when people want to know how many.'],
		donts: ['Type it in capitals.', 'Use capitals anywhere else.'],
		content: ['One or two words: "Sources", "Follow up".'],
		accessibility: ['A paragraph, not a heading. Screen readers read the words as written, not in capitals.'],
		related: ['navigation/NavGroupLabel.svelte', 'status/Tag.svelte'],
	},
	{
		file: 'status/Status.svelte',
		summary:
			'A state shown as a 6px dot plus a word, so colour never carries the meaning alone: live, ready, idle, failed, needs attention, or working, where the agent\'s mascot takes the dot\'s place.',
		use: ['Whether a project or build is live, ready, idle or failed.', 'Who is working right now: "Koda is building".'],
		avoid: [
			'A verdict or marker on an item (go, no-go, draft): use Tag.',
			'The step an agent is on, with the time so far: use LiveStep.',
		],
		dos: ['Always pair the dot with a word: "Live", "Deploy failed", "Needs attention".', 'Use working with the agent who is doing the work.'],
		donts: ['Pulse or animate the dot: these are states, not decoration.', 'A dot on its own.'],
		content: [
			'Live, failed, attention and working take a sentence-case word or phrase: "Live", "Deploy failed", "Koda is building".',
			'Ready and idle are one lower-case word in mono: "ready", "idle".',
		],
		accessibility: ['The words carry the state; the dot and the mascot are decorative.', 'It is not a live region: a change is not announced on its own.'],
		related: ['status/Tag.svelte', 'conversation/LiveStep.svelte', 'lists/ProjectRow.svelte'],
	},
	{
		file: 'status/Tag.svelte',
		summary:
			'A go, no-go or standby verdict in small mono capitals, 18px tall. Also used for short markers on cards, such as "Recommended", "Draft", "Needs you" and "Next".',
		use: [
			'go: the recommended or approved option ("Recommended").',
			'no-go: something that blocks.',
			'neutral: a quiet marker ("Draft", "Default", "Needs you", "Next").',
		],
		avoid: ['Whether something is live or failed: use Status.', 'Something people can pick: use Chip.', 'A group\'s name: use SectionLabel.'],
		dos: ['Write the label in sentence case; the capitals come from CSS.', 'Keep it to a word or two.'],
		donts: ['Type it in capitals.', 'Put a sentence in a tag.'],
		content: ['One or two words in sentence case in the code: "No-go", "Needs you".'],
		accessibility: ['Plain text; screen readers read the words as written. The tint never carries the meaning alone.'],
		related: ['status/Status.svelte', 'status/SectionLabel.svelte', 'inputs/Chip.svelte'],
	},
] satisfies ComponentDoc[];
