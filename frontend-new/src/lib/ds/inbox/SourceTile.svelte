<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Inbox (BFoiZ) → Source Tile: whose an entry is, as the crew member's tile with their
	// mascot at 20. 32 with corners of 8 in the feed; 26 with 6.5 in Coming up's agenda.
	type Props = { agent: CrewId; size?: 32 | 26 };
	let { agent, size = 32 }: Props = $props();
</script>

<span
	data-source-tile
	class={['inline-flex shrink-0 items-center justify-center', size === 32 ? 'size-8 rounded-inner' : 'size-6.5 rounded-agenda']}
	style="background: var(--crew-{agent}-bg)"
>
	<Mascot {agent} size={20} />
</span>
