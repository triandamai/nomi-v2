<script lang="ts" module>
	import type { CrewId } from '$lib/ds/agents/crew';

	export type LandingAgent = CrewId;
</script>

<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';

	// The landing page's crew wear the app's own mascots (agents/Mascot.svelte: each member's agent
	// shape, gradient and two-dot face, brought to life by open-bot-avatar, as in v2), not the Figma
	// file's flat drawings, so the page shows the crew people meet once they're in. The page needs
	// sizes Mascot doesn't offer (24, 44, 68, 150…), and Mascot's props stay as the old app's twin
	// has them, so it's drawn at 96 and filled to this box: the flat SVG and the live engine both
	// scale to their host (the engine paints a viewBox at 100% and watches its size), and the engine
	// pauses itself offscreen. Live from 24 up; below that it stays flat, as Mascot does at 16.
	// Decorative unless given a label.
	type Props = {
		agent: LandingAgent;
		/** Its square, in px. */
		size: number;
		label?: string;
		/** Hop in the page's 6s loop, starting this many seconds in. */
		hop?: number;
	};
	let { agent, size, label, hop }: Props = $props();
</script>

<span
	class={['mascot', hop !== undefined && 'hop']}
	style:width="{size}px"
	style:height="{size}px"
	style:--lp-delay={hop !== undefined ? `${hop}s` : undefined}
	role={label ? 'img' : undefined}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
>
	<Mascot {agent} size={96} live={size >= 24} />
</span>

<style>
	.mascot {
		position: relative;
		display: inline-flex;
		flex-shrink: 0;
	}

	/* Mascot at 96, filled to this box: its live wrapper, the flat SVG inside it, or the SVG alone. */
	.mascot > :global(span),
	.mascot > :global(svg),
	.mascot > :global(span > svg) {
		width: 100% !important;
		height: 100% !important;
	}

	/* lp-hop is landing.css's: two hops in the page's 6s loop. */
	.hop {
		transform-origin: 50% 100%;
		animation: lp-hop 6s infinite;
		animation-delay: var(--lp-delay, 0s);
	}
</style>
