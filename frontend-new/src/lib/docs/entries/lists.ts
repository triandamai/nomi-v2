import type { ComponentDoc } from '../types';

export default [
	{
		file: 'lists/EmptyState.svelte',
		summary: 'What a list or panel shows when there is nothing in it yet: its title and count, a plain sentence saying so, and the one thing to do about it.',
		use: ['Any list or panel that can be empty, such as Reminders. Empty, loading, error and populated states ship together.'],
		avoid: ['While the list is loading: use SkeletonRow.', 'When loading failed: say what went wrong instead of showing it as empty.'],
		dos: ['Say what is missing and who can help: "Nothing scheduled today. Ask Kala, or add one yourself."', 'Offer one action, as a link: "Add reminder".'],
		donts: ['Illustrations or emoji.', 'More than one action.'],
		content: ['The title is the list\'s name: "Reminders".', 'The body is one or two plain sentences.', 'The action names what it adds: "Add reminder".'],
		accessibility: ['A section with a header; the icon is decorative.', 'The action is a real link.'],
		related: ['lists/SkeletonRow.svelte', 'actions/Link.svelte'],
	},
	{
		file: 'lists/ProjectRow.svelte',
		summary:
			'A project in a list: its app tile in a crew tint with the initial in the app\'s serif, its name and description, whether it is live and where, and when it last changed. The whole row is a link.',
		use: ['Lists of projects.'],
		avoid: ['Recent projects in the sidebar: use RecentRow.', 'Something still loading: use SkeletonRow until the row has real data.'],
		dos: ['Show the status in words: "Live", "Deploy failed".', 'Show the live URL in full, in mono.', 'Keep the time short: "2d".'],
		donts: ['Use the tile\'s tint anywhere else in the row: only the app\'s identity tile takes a crew colour.'],
		content: [
			'The name exactly as the project is called: "salah-daily-tracker".',
			'The description is one plain line about what it does: "Daily salah tracker with countdown and streaks".',
		],
		accessibility: ['One link for the whole row.', 'On small screens the status and URL column is hidden, so the name and description must stand on their own.'],
		related: ['navigation/RecentRow.svelte', 'status/Status.svelte', 'lists/SkeletonRow.svelte'],
	},
	{
		file: 'lists/SkeletonRow.svelte',
		summary: 'Stands in while a list loads, and says what is loading ("loading 4 projects…"). It stays still: no shimmer, no fake progress.',
		use: ['The first load of a list or panel.'],
		avoid: ['Work an agent is doing: use LiveStep.', 'A list that has loaded with nothing in it: use EmptyState.'],
		dos: ['Say what is loading, and how many when you know.'],
		donts: ['Add a shimmer or a pulse.', 'Leave it up after the load fails: show the error.'],
		content: ['Lower case, in mono, ending with an ellipsis: "loading 4 projects…".'],
		accessibility: ['role="status" with aria-busy="true"; the grey shapes are hidden from screen readers and the label is read.'],
		related: ['lists/EmptyState.svelte', 'lists/ProjectRow.svelte'],
	},
	{
		file: 'lists/SuggestionRow.svelte',
		summary:
			'Something people could ask, as a full-width row they can press, with the mascot of the crew member it would go to and the route in mono ("→ dana").',
		use: ['Ideas for what to ask, where a conversation starts.'],
		avoid: ['Answers to a question just asked: use QuickReplies.', 'Next questions under an answer: use FollowUps.'],
		dos: ['Write it as the person would ask it: "Summarize this week\'s spending".', 'Show who it goes to with the route.'],
		donts: ['Tint the row in the crew colour at rest: the crew-colour board allows suggestions a tint only on hover.'],
		content: ['A request in sentence case, no full stop.', 'The route is "→" and the member\'s name in lower case.'],
		accessibility: ['A real button covering the whole row; the mascot is decorative and the route is read as text.'],
		related: ['chat/QuickReplies.svelte', 'chat/FollowUps.svelte', 'agents/Mascot.svelte'],
	},
] satisfies ComponentDoc[];
