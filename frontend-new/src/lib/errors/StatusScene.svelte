<script lang="ts">
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import type { CrewId } from '$lib/ds/agents/crew';

	// The crew acting out a status page (the old app's StatusScene, with v2's mascots): Nomi in the
	// middle, four of the crew around it.
	//  - lost (404): Nomi looks around, the crew bounce in turns looking for the page, one peeks in.
	//  - broken (500): Nomi wobbles, the crew rush in to help and bounce back, sparks fly off.
	// Decorative (the page says what happened); reduced motion stops all of it.
	let { variant }: { variant: 'lost' | 'broken' } = $props();

	const CREW: CrewId[] = ['koda', 'dana', 'kala', 'rena'];
</script>

<div class="relative mx-auto h-55 w-full max-w-90" data-scene={variant} aria-hidden="true">
	<div class="absolute bottom-4 h-2.5 rounded-full bg-shade blur-xs" data-part="ground"></div>
	<div class="absolute bottom-6 left-1/2 -translate-x-1/2" data-part="nomi">
		<Mascot agent="nomi" size={96} mood={variant === 'broken' ? 'thinking' : 'idle'} />
	</div>
	{#each CREW as agent, index (agent)}
		<div class="absolute size-13" data-part="mate" data-mate={index}>
			<span class="absolute top-0 left-0 origin-top-left" data-part="small"><Mascot {agent} size={96} mood={variant === 'broken' ? 'thinking' : 'happy'} /></span>
		</div>
	{/each}
	{#if variant === 'lost'}
		<span class="absolute top-0 left-1/2 -ml-3 w-6 text-center font-mono text-display font-medium text-accent" data-part="mark">?</span>
	{:else}
		<span class="absolute top-10 size-2.5 rounded-2xs bg-warning" data-part="spark" data-spark="a"></span>
		<span class="absolute top-13 size-2.5 rounded-2xs bg-danger" data-part="spark" data-spark="b"></span>
	{/if}
</div>

<style>
	[data-part='ground'] {
		right: 8%;
		left: 8%;
	}
	/* The crew around Nomi are drawn at 96 and shown at 52. */
	[data-part='small'] {
		scale: 0.54;
	}
	[data-mate='0'] {
		bottom: 24px;
		left: 2%;
	}
	[data-mate='1'] {
		bottom: 116px;
		left: 18%;
	}
	[data-mate='2'] {
		bottom: 116px;
		right: 18%;
	}
	[data-mate='3'] {
		bottom: 24px;
		right: 2%;
	}
	[data-spark='a'] {
		left: 44%;
	}
	[data-spark='b'] {
		left: 58%;
	}

	@media (prefers-reduced-motion: no-preference) {
		/* Lost: Nomi looks around; the crew bounce in turns, searching; one peeks in and out. */
		[data-scene='lost'] [data-part='nomi'] {
			animation: look 3.2s ease-in-out infinite;
		}
		[data-scene='lost'] [data-part='mate'] {
			animation: bounce 1.6s cubic-bezier(0.3, 0, 0.4, 1) infinite;
		}
		[data-scene='lost'] [data-mate='0'] {
			animation: peek 4s ease-in-out infinite;
		}
		[data-scene='lost'] [data-mate='1'] {
			animation-delay: -0.4s;
		}
		[data-scene='lost'] [data-mate='2'] {
			animation-delay: -0.8s;
		}
		[data-scene='lost'] [data-mate='3'] {
			animation-delay: -1.2s;
		}
		[data-scene='lost'] [data-part='mark'] {
			animation: pulse 1.6s ease-in-out infinite;
		}

		/* Broken: Nomi wobbles; the crew hurry over and back, bouncing; sparks fly off. */
		[data-scene='broken'] [data-part='nomi'] {
			animation: wobble 1.4s ease-in-out infinite;
		}
		[data-scene='broken'] [data-mate='0'] {
			animation: rush-right 2.4s ease-in-out infinite;
		}
		[data-scene='broken'] [data-mate='3'] {
			animation: rush-left 2.4s ease-in-out infinite;
			animation-delay: -1.2s;
		}
		[data-scene='broken'] [data-mate='1'],
		[data-scene='broken'] [data-mate='2'] {
			animation: bounce 1.2s cubic-bezier(0.3, 0, 0.4, 1) infinite;
		}
		[data-scene='broken'] [data-mate='2'] {
			animation-delay: -0.6s;
		}
		[data-scene='broken'] [data-part='spark'] {
			animation: fly 1.4s ease-out infinite;
		}
		[data-scene='broken'] [data-spark='b'] {
			animation-delay: -0.7s;
		}
	}

	@keyframes look {
		0%,
		100% {
			rotate: -6deg;
		}
		50% {
			rotate: 6deg;
		}
	}
	@keyframes bounce {
		0%,
		55%,
		100% {
			translate: 0 0;
			scale: 1 1;
		}
		20% {
			translate: 0 -18px;
			scale: 0.96 1.04;
		}
		45% {
			translate: 0 0;
			scale: 1.06 0.94;
		}
	}
	@keyframes peek {
		0%,
		100% {
			translate: -40px 0;
			opacity: 0;
		}
		30%,
		70% {
			translate: 0 0;
			opacity: 1;
		}
		45% {
			translate: 0 -12px;
		}
	}
	@keyframes pulse {
		0%,
		100% {
			scale: 1;
		}
		50% {
			scale: 1.2;
		}
	}
	@keyframes wobble {
		0%,
		100% {
			rotate: 0deg;
		}
		25% {
			rotate: -8deg;
		}
		75% {
			rotate: 8deg;
		}
	}
	@keyframes rush-right {
		0%,
		100% {
			translate: 0 0;
		}
		25% {
			translate: 24px -14px;
		}
		50% {
			translate: 46px 0;
		}
		75% {
			translate: 24px -10px;
		}
	}
	@keyframes rush-left {
		0%,
		100% {
			translate: 0 0;
		}
		25% {
			translate: -24px -14px;
		}
		50% {
			translate: -46px 0;
		}
		75% {
			translate: -24px -10px;
		}
	}
	@keyframes fly {
		from {
			translate: 0 0;
			opacity: 1;
		}
		to {
			translate: 0 -40px;
			opacity: 0;
			rotate: 90deg;
		}
	}
</style>
