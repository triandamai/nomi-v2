<script lang="ts">
	import type { Snippet } from 'svelte';
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Landing Page → Option 3 — Hero Nomi (14:383) → Hero (14:593) and Composer stage (14:601):
	// the brand glow (760 × 420) behind it all; Nomi's 112px mascot, a speech bubble and the greeting
	// in Instrument Serif, 14 apart and centred; then the composer, 40 under the top of its stage,
	// with crew members peeking over its top edge (52px, tilted as drawn: 8°, −6°, 6°, −10°). The
	// peeks sit behind the composer, so only their heads show. A phone leaves the peeks out and sets
	// the greeting at 40.
	// The peeks wear each member's own mascot (the agent shapes, AGENTS.md §9), only for members the
	// person has: who's drawn is decoration of who's on the crew, never a button.
	type Props = { title: string; bubble: string; peeks?: CrewId[]; composer: Snippet };
	let { title, bubble, peeks = [], composer }: Props = $props();

	// Where each peek sits along the stage's 760 width, and its tilt, in the design's order.
	const SPOTS = [
		{ side: 'left-13', tilt: 'rotate-8' },
		{ side: 'left-35', tilt: '-rotate-6' },
		{ side: 'right-38', tilt: 'rotate-6' },
		{ side: 'right-15', tilt: '-rotate-10' },
	];
</script>

<div class="relative flex w-full flex-col items-center">
	<div aria-hidden="true" class="pointer-events-none absolute -top-12 left-1/2 h-105 w-full max-w-190 -translate-x-1/2 bg-brand-glow"></div>
	<div class="relative flex flex-col items-center gap-3.5 text-center">
		<span class="max-md:hidden"><Mascot agent="nomi" size={112} /></span>
		<span class="md:hidden"><Mascot agent="nomi" size={72} /></span>
		<p class="rounded-xl bg-surface px-3.5 py-2 text-ui text-text-2 outline -outline-offset-1 outline-border">{bubble}</p>
		<h1 class="font-serif text-greeting-sm font-normal text-text md:text-greeting">{title}</h1>
	</div>
	<div class="relative mt-8 w-full max-w-190 md:mt-2 md:pt-10">
		{#each peeks.slice(0, SPOTS.length) as agent, index (agent)}
			<span aria-hidden="true" class={['absolute top-0 max-md:hidden', SPOTS[index].side, SPOTS[index].tilt]}><Mascot {agent} size={52} /></span>
		{/each}
		<div class="relative">{@render composer()}</div>
	</div>
</div>
