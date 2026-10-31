import type { ComponentDoc } from '../types';

export default [
	{
		file: 'agents/Mascot.svelte',
		summary:
			'A crew member\'s mascot: one flat colour and one silhouette per member, at 96, 32, 20 or 16px. Identity, not decoration: it appears only where that member speaks or works.',
		use: [
			'20px beside a name: bylines, chips, suggestions, who is working.',
			'32px in lists of the crew.',
			'96px where a member is introduced on their own.',
		],
		avoid: ['A person: use Avatar.', 'Decorating an empty state or a page.'],
		dos: ['Show the mascot of the member who is actually speaking or working.', 'Give it a label when it stands without the name.'],
		donts: [
			'Head accessories, gradients or shading.',
			'Sizes other than 96, 32, 20 and 16.',
			'Spread the member\'s colour onto the rest of the interface: it belongs only on their own surfaces.',
		],
		accessibility: ['Hidden from screen readers by default, because the name sits beside it. With label it becomes role="img" with that name.'],
		related: ['identity/AgentByline.svelte', 'identity/CrewRow.svelte', 'identity/Avatar.svelte'],
	},
] satisfies ComponentDoc[];
