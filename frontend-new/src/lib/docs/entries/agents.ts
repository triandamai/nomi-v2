import type { ComponentDoc } from '../types';

export default [
	{
		file: 'agents/Mascot.svelte',
		summary:
			'A crew member\'s agent shape: their own outline in their own gradient, with the two-dot face. At 96 and 32 it is live (open-bot-avatar) with volume, blinking, gaze tracking and moods.',
		use: [
			'20px beside a name: bylines, chips, suggestions, who is working (flat).',
			'32px in lists of the crew (live).',
			'96px where a member is introduced on their own (live).',
			'mood="thinking" while that member works, "happy" when they finish.',
		],
		avoid: ['A person: use Avatar.', 'Decorating an empty state or a page.'],
		dos: ['Show the mascot of the member who is actually speaking or working.', 'Give it a label when it stands without the name.'],
		donts: [
			'Head accessories, shading, or a gradient other than the member\'s own.',
			'Sizes other than 96, 32, 20 and 16.',
			'Spread the member\'s colours onto the rest of the interface: it belongs only on their own surfaces.',
		],
		accessibility: [
			'Hidden from screen readers by default, because the name sits beside it. With label it becomes role="img" with that name.',
			'The live mascot follows prefers-reduced-motion: it holds a calm pose and stops blinking; only its gaze still moves.',
		],
		related: ['identity/AgentByline.svelte', 'identity/CrewRow.svelte', 'identity/Avatar.svelte'],
	},
] satisfies ComponentDoc[];
