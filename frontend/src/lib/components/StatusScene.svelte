<script lang="ts">
	// The crew acting out a status page: Nomi in the middle, the others around it.
	//  - lost (404): they look around for the page, one peeks in from the side.
	//  - broken (500): Nomi wobbles, the crew rushes in to help.
	//  - maintenance: they work on something together, taking turns.
	// Motion stops for people who ask for reduced motion.
	import AgentShape from '$lib/components/m3/AgentShape.svelte';

	let { variant }: { variant: 'lost' | 'broken' | 'maintenance' } = $props();

	const CREW = ['coding', 'money', 'reminders', 'planning'];
</script>

<div class="scene scene--{variant}" aria-hidden="true">
	<div class="scene__ground"></div>
	<div class="scene__nomi"><AgentShape size={112} face working={variant !== 'lost'} /></div>
	{#each CREW as agent, i (agent)}
		<div class="scene__mate scene__mate--{i}"><AgentShape {agent} size={52} face working={variant === 'maintenance'} /></div>
	{/each}
	{#if variant === 'lost'}
		<span class="scene__mark">?</span>
	{:else if variant === 'broken'}
		<span class="scene__spark scene__spark--a"></span>
		<span class="scene__spark scene__spark--b"></span>
	{:else}
		<span class="scene__gear"></span>
	{/if}
</div>

<style>
	.scene {
		position: relative;
		width: min(360px, 86vw);
		height: 220px;
		margin: 0 auto;
	}
	.scene__ground {
		position: absolute;
		right: 8%;
		bottom: 18px;
		left: 8%;
		height: 10px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 10%, transparent);
		filter: blur(2px);
	}
	.scene__nomi {
		position: absolute;
		bottom: 26px;
		left: 50%;
		translate: -50% 0;
	}
	.scene__mate {
		position: absolute;
		bottom: 26px;
	}
	.scene__mate--0 {
		left: 4%;
	}
	.scene__mate--1 {
		left: 20%;
		bottom: 112px;
	}
	.scene__mate--2 {
		right: 20%;
		bottom: 112px;
	}
	.scene__mate--3 {
		right: 4%;
	}
	.scene__mark {
		position: absolute;
		top: 4px;
		left: 50%;
		translate: -50% 0;
		color: var(--md-sys-color-primary);
		font-family: var(--md-ref-typeface-brand);
		font-size: 2.5rem;
		font-weight: 800;
	}
	.scene__spark {
		position: absolute;
		width: 10px;
		height: 10px;
		border-radius: 2px;
		background: var(--md-sys-color-tertiary);
	}
	.scene__spark--a {
		top: 40px;
		left: 44%;
	}
	.scene__spark--b {
		top: 52px;
		left: 58%;
		background: var(--md-sys-color-error);
	}
	.scene__gear {
		position: absolute;
		top: 18px;
		left: 50%;
		width: 34px;
		height: 34px;
		margin-left: -17px;
		border: 6px dotted var(--md-sys-color-primary);
		border-radius: 50%;
	}

	@media (prefers-reduced-motion: no-preference) {
		/* Lost: everyone looks around, one peeks in and out. */
		.scene--lost .scene__nomi {
			animation: look 3.2s ease-in-out infinite;
		}
		.scene--lost .scene__mate--0 {
			animation: peek 4s ease-in-out infinite;
		}
		.scene--lost .scene__mate--1,
		.scene--lost .scene__mate--2 {
			animation: float 3s ease-in-out infinite;
		}
		.scene--lost .scene__mate--2 {
			animation-delay: -1.5s;
		}
		.scene--lost .scene__mate--3 {
			animation: look 3.2s ease-in-out infinite reverse;
		}
		.scene--lost .scene__mark {
			animation: pulse 1.6s ease-in-out infinite;
		}

		/* Broken: Nomi wobbles, the crew hurries over and back. */
		.scene--broken .scene__nomi {
			animation: wobble 1.4s ease-in-out infinite;
		}
		.scene--broken .scene__mate--0 {
			animation: rush-right 2.4s ease-in-out infinite;
		}
		.scene--broken .scene__mate--3 {
			animation: rush-left 2.4s ease-in-out infinite;
			animation-delay: -1.2s;
		}
		.scene--broken .scene__mate--1,
		.scene--broken .scene__mate--2 {
			animation: float 1.8s ease-in-out infinite;
		}
		.scene--broken .scene__spark {
			animation: fly 1.4s ease-out infinite;
		}
		.scene--broken .scene__spark--b {
			animation-delay: -0.7s;
		}

		/* Maintenance: they hop in turns while the gear turns. */
		.scene--maintenance .scene__mate {
			animation: hop 2s ease-in-out infinite;
		}
		.scene--maintenance .scene__mate--1 {
			animation-delay: -0.5s;
		}
		.scene--maintenance .scene__mate--2 {
			animation-delay: -1s;
		}
		.scene--maintenance .scene__mate--3 {
			animation-delay: -1.5s;
		}
		.scene--maintenance .scene__nomi {
			animation: float 3s ease-in-out infinite;
		}
		.scene--maintenance .scene__gear {
			animation: spin 4s linear infinite;
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
	}
	@keyframes float {
		0%,
		100% {
			translate: 0 0;
		}
		50% {
			translate: 0 -10px;
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
		50% {
			translate: 46px -8px;
		}
	}
	@keyframes rush-left {
		0%,
		100% {
			translate: 0 0;
		}
		50% {
			translate: -46px -8px;
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
	@keyframes hop {
		0%,
		60%,
		100% {
			translate: 0 0;
		}
		30% {
			translate: 0 -18px;
		}
	}
	@keyframes spin {
		to {
			rotate: 360deg;
		}
	}
</style>
