<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Settings / Crew (vVUeh) → Custom Mark: who a member is at a glance. A built-in member
	// wears their mascot; a custom one (or one the pen doesn't draw) its initial in mono, in a ring.
	type Props = { agent?: CrewId; initial?: string; size?: 20 | 32 };
	let { agent, initial = '', size = 20 }: Props = $props();
</script>

{#if agent}
	<Mascot {agent} {size} />
{:else}
	<span
		data-part="mark"
		aria-hidden="true"
		class={['inline-flex shrink-0 items-center justify-center rounded-full font-mono text-text outline -outline-offset-1 outline-border-strong', size === 32 ? 'size-8 text-ui' : 'size-5 text-label']}
		>{initial}</span
	>
{/if}
