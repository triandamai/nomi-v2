import type { ComponentDoc } from '../types';

export default [
	{
		file: 'identity/AgentByline.svelte',
		summary:
			'Who is speaking and when: the crew member\'s 20px mascot, their name, and mono meta such as "15:37 · 4m 12s" or "14:02 · deepseek-flash". It heads every agent message.',
		use: ['At the top of an agent\'s message.', 'Wherever work is credited to one crew member.'],
		avoid: [
			'A person: use Avatar.',
			'A crew member in a list, with what they do: use CrewRow.',
			'Showing that someone is working right now: use Status (working).',
		],
		dos: ['Put the time first in the meta, then the model or how long it took.', 'Use the name of the member who actually answered.'],
		donts: ['Colour the name in the member\'s crew colour: the mascot already carries it.', 'Show the mascot without the name.'],
		content: ['The name as the crew is called: "Koda", "Nomi".', 'Meta in mono, parts joined with " · ": "14:02 · deepseek-flash".'],
		accessibility: ['The mascot is decorative here, because the name sits beside it.'],
		related: ['conversation/MessageAgent.svelte', 'agents/Mascot.svelte', 'identity/CrewRow.svelte', 'identity/Avatar.svelte'],
	},
	{
		file: 'identity/Avatar.svelte',
		summary: 'A person: their picture, or their initials in mono on surface-2, in a 28px rounded square with a border-strong edge.',
		use: ['The signed-in person at the foot of the sidebar and the settings nav.', 'Anyone else who is a person, not a crew member.'],
		avoid: ['A crew member: use Mascot.', 'A teammate in a shared project, in their peer colour: use PeerAvatar.'],
		dos: ['Pass the full name; the initials come from its first two words.'],
		donts: ['Use an avatar for an agent, or a mascot for a person.'],
		accessibility: ['A picture has the name as its alt text; the initials version is role="img" named with the full name.'],
		related: ['agents/Mascot.svelte', 'chrome/UserRow.svelte', 'build/PeerAvatar.svelte', 'identity/AgentByline.svelte'],
	},
	{
		file: 'identity/CrewRow.svelte',
		summary:
			'A crew member in a list: their 32px mascot, their name, what they look after, and an optional state in mono ("idle"). Given an href, the whole row is a link.',
		use: ['Lists of the crew, such as the crew in settings or a panel of who is around.'],
		avoid: ['Crediting a message: use AgentByline.', 'A person: use Avatar beside their name.'],
		dos: ['Say in a few words what the member handles: "Transactions, budgets, subscriptions".', 'Show the state only when it is real.'],
		donts: ['Tint the row in the crew colour: the mascot carries the identity.'],
		content: ['The role is a short phrase with no full stop: "Reminders on time", "Reads attachments".', 'The state is one lower-case word in mono: "idle".'],
		accessibility: ['The mascot is decorative; the name says who it is.', 'With an href it is a single link, so the whole row is one stop for the keyboard.'],
		related: ['identity/AgentByline.svelte', 'agents/Mascot.svelte', 'status/Status.svelte'],
	},
] satisfies ComponentDoc[];
