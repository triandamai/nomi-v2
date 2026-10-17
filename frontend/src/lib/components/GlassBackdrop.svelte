<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/state';
	import { rosterKey } from '$lib/crew';
	import { agentLook, GRADIENT_STOPS, shapePath } from '$lib/components/m3/shapes';

	// The frosted stage behind a regular chat and the Stage home: the theme's action gradient as
	// soft blurred light, Nomi and the crew drifting in it, and a translucent veil on top so text
	// stays readable. Panes laid over it (app bar, composer) frost it with backdrop-filter.
	//
	// Kept cheap on purpose (measured: a live CSS blur on the light pools took the chat from 60
	// to 8 fps). The shapes' blur is part of their SVG drawing, so each is painted once and the
	// compositor only moves the result; a CSS `filter` on a moving layer would be re-run every
	// frame. The light pools are soft radial gradients that hold still. Every animation is
	// transform-only, the layer is `contain: strict`, it holds still under reduced motion, on
	// low-power devices and with Save-Data, and it pauses while the tab is hidden or the stage
	// is scrolled out of view.

	let {
		agents,
		withoutNomi = false,
		busy = false,
	}: {
		/** Who drifts on the stage, Nomi first; defaults to the person's crew. */
		agents?: string[];
		/** Leave Nomi out (the page already shows Nomi's mark up front). */
		withoutNomi?: boolean;
		/** The crew is working: the light warms up a little. */
		busy?: boolean;
	} = $props();

	// Where each shape sits, how big, how far back (far ones are blurred and faint), and how it
	// moves. Positions keep the composer's corner and the centre column mostly clear.
	// Blur (SVG units of the 48-unit shape) for each depth: soft enough that the crew reads as
	// coloured light, not as figures.
	const BLUR = { near: 4.5, mid: 5.5, far: 6.5 } as const;
	const SLOTS: { x: number; y: number; size: number; depth: 'near' | 'mid' | 'far'; delay: number }[] = [
		{ x: 78, y: 16, size: 132, depth: 'near', delay: 0 },
		{ x: 8, y: 58, size: 96, depth: 'mid', delay: -2.4 },
		{ x: 22, y: 12, size: 64, depth: 'far', delay: -5.1 },
		{ x: 88, y: 64, size: 84, depth: 'mid', delay: -1.3 },
		{ x: 52, y: 82, size: 56, depth: 'far', delay: -3.7 },
		{ x: 60, y: 30, size: 44, depth: 'far', delay: -6.2 },
	];

	const crewKeys = $derived.by(() => {
		const roster = (page.data.crew ?? []) as { agent_type: string }[];
		const keys = agents ?? ['nomi', ...roster.map((member) => rosterKey(member.agent_type)).filter((key) => key !== 'nomi')];
		return [...new Set(keys)].filter((key) => !(withoutNomi && key === 'nomi')).slice(0, SLOTS.length);
	});
	const shapes = $derived(
		crewKeys.map((key, i) => {
			const look = agentLook(key);
			return { key, slot: SLOTS[i], motion: look.motion, path: shapePath(look.shape), stops: GRADIENT_STOPS[look.tone] };
		}),
	);

	const uid = $props.id();
	let stage: HTMLDivElement | undefined = $state();
	// Off when the device asks for less: reduced motion, Save-Data, or a small device.
	let still = $state(false);
	let paused = $state(false);

	onMount(() => {
		const nav = navigator as Navigator & { connection?: { saveData?: boolean }; deviceMemory?: number };
		const lowPower =
			nav.connection?.saveData === true ||
			(nav.deviceMemory !== undefined && nav.deviceMemory <= 2) ||
			(nav.hardwareConcurrency !== undefined && nav.hardwareConcurrency <= 2);
		const reduced = window.matchMedia('(prefers-reduced-motion: reduce)');
		const update = () => (still = lowPower || reduced.matches);
		update();
		reduced.addEventListener('change', update);

		// Nothing animates while nobody can see it.
		let visible = true;
		let onScreen = true;
		const sync = () => (paused = !visible || !onScreen);
		const onVisibility = () => {
			visible = document.visibilityState === 'visible';
			sync();
		};
		document.addEventListener('visibilitychange', onVisibility);
		const observer = new IntersectionObserver(([entry]) => {
			onScreen = entry.isIntersecting;
			sync();
		});
		if (stage) observer.observe(stage);

		return () => {
			reduced.removeEventListener('change', update);
			document.removeEventListener('visibilitychange', onVisibility);
			observer.disconnect();
		};
	});
</script>

<div bind:this={stage} class="glass" class:glass--still={still} class:glass--paused={paused} class:glass--busy={busy} aria-hidden="true">
	<span class="glass__light glass__light--a"></span>
	<span class="glass__light glass__light--b"></span>
	<span class="glass__light glass__light--c"></span>
	{#each shapes as { key, slot, motion, path, stops }, i (key)}
		<span
			class="glass__drift glass__drift--{slot.depth}"
			style:left="{slot.x}%"
			style:top="{slot.y}%"
			style:--size="{slot.size}px"
			style:--delay="{slot.delay}s"
			style:--drift-duration="{11 + i * 1.7}s"
		>
			<span class="glass__move" data-motion={motion}>
				<svg viewBox="-16 -16 80 80" class="glass__shape">
					<defs>
						<linearGradient id="{uid}-g{i}" x1="0" y1="0" x2="1" y2="1">
							{#each stops as stop, s (s)}
								<stop offset={stops.length === 1 ? 0 : s / (stops.length - 1)} style="stop-color: {stop}" />
							{/each}
						</linearGradient>
						<filter id="{uid}-b{i}" x="-50%" y="-50%" width="200%" height="200%">
							<feGaussianBlur stdDeviation={BLUR[slot.depth]} />
						</filter>
					</defs>
					<path d={path} fill="url(#{uid}-g{i})" filter="url(#{uid}-b{i})" />
				</svg>
			</span>
		</span>
	{/each}
	<span class="glass__veil"></span>
</div>

<style>
	.glass {
		position: absolute;
		inset: 0;
		z-index: 0;
		overflow: hidden;
		pointer-events: none;
		/* Nothing inside affects layout or paint outside, so the browser can skip it entirely. */
		contain: strict;
		container: glass / inline-size;
		background: var(--nomi-color-ground);
	}

	/* Light: three big soft pools of the theme's action gradient. */
	.glass__light {
		position: absolute;
		width: 62vmax;
		height: 62vmax;
		border-radius: 50%;
		opacity: var(--glass-light, 0.5);
	}
	.glass__light--a {
		top: -24vmax;
		left: -16vmax;
		background: radial-gradient(closest-side, var(--nomi-action-1), transparent);
	}
	.glass__light--b {
		top: 8%;
		right: -26vmax;
		background: radial-gradient(closest-side, var(--nomi-action-2), transparent);
	}
	.glass__light--c {
		bottom: -30vmax;
		left: 18%;
		background: radial-gradient(closest-side, var(--nomi-action-3), transparent);
	}
	.glass__light {
		transition: opacity 1.2s ease;
	}
	.glass--busy .glass__light {
		opacity: calc(var(--glass-light, 0.5) * 1.35);
	}
	/* Dark grounds take less light before it washes the text out. */
	:global(:root[data-theme='dark']) .glass {
		--glass-light: 0.32;
	}
	@media (prefers-color-scheme: dark) {
		:global(:root:not([data-theme='light']):not([data-theme='dark'])) .glass {
			--glass-light: 0.32;
		}
	}

	/* Crew: each shape floats on its own path (outer) and moves in its agent's motion (inner). */
	.glass__drift {
		position: absolute;
		--scaled: calc(var(--size) * var(--glass-scale, 1));
		width: var(--scaled);
		height: var(--scaled);
		margin: calc(var(--scaled) / -2) 0 0 calc(var(--scaled) / -2);
		will-change: transform;
		animation: glass-drift var(--drift-duration) ease-in-out var(--delay) infinite alternate;
	}
	/* Barely there: the crew is felt more than seen. */
	.glass__drift--far {
		opacity: 0.3;
	}
	.glass__drift--mid {
		opacity: 0.38;
	}
	.glass__drift--near {
		opacity: 0.46;
	}
	.glass__shape {
		display: block;
		width: 100%;
		height: 100%;
		overflow: visible;
	}
	/* Narrow screens: smaller shapes, so they stay in the margins rather than under the text. */
	@container glass (max-width: 640px) {
		.glass__drift {
			--glass-scale: 0.55;
		}
	}
	.glass__move {
		display: block;
		width: 100%;
		height: 100%;
		transform-origin: 50% 50%;
		will-change: transform;
		animation-delay: var(--delay);
		animation-iteration-count: infinite;
	}
	.glass__move[data-motion='spin'] {
		animation-name: glass-spin;
		animation-duration: 22s;
		animation-timing-function: linear;
	}
	.glass__move[data-motion='orbit'] {
		animation-name: glass-orbit;
		animation-duration: 16s;
		animation-timing-function: linear;
	}
	.glass__move[data-motion='bounce'] {
		animation-name: glass-bounce;
		animation-duration: 2.8s;
		animation-timing-function: cubic-bezier(0.38, 1.21, 0.22, 1);
	}
	.glass__move[data-motion='wobble'] {
		animation-name: glass-wobble;
		animation-duration: 5s;
		animation-timing-function: ease-in-out;
	}
	.glass__move[data-motion='pulse'] {
		animation-name: glass-pulse;
		animation-duration: 4.2s;
		animation-timing-function: cubic-bezier(0.38, 1.21, 0.22, 1);
	}

	/* Veil: the frosted pane between the stage and the conversation. */
	.glass__veil {
		position: absolute;
		inset: 0;
		background: color-mix(in srgb, var(--nomi-color-ground) 58%, transparent);
	}

	.glass--paused :is(.glass__drift, .glass__move) {
		animation-play-state: paused;
	}
	.glass--still :is(.glass__drift, .glass__move) {
		animation: none;
		will-change: auto;
	}

	@keyframes glass-drift {
		from {
			transform: translate3d(-14px, 10px, 0);
		}
		to {
			transform: translate3d(16px, -18px, 0);
		}
	}
	@keyframes glass-spin {
		to {
			transform: rotate(360deg);
		}
	}
	@keyframes glass-orbit {
		0% {
			transform: rotate(0deg) translateX(10%) rotate(0deg);
		}
		100% {
			transform: rotate(360deg) translateX(10%) rotate(-360deg);
		}
	}
	@keyframes glass-bounce {
		0%,
		100% {
			transform: translateY(0) scale(1, 1);
		}
		30% {
			transform: translateY(-16%) scale(0.95, 1.05);
		}
		55% {
			transform: translateY(0) scale(1.07, 0.92);
		}
		70% {
			transform: translateY(0) scale(1, 1);
		}
	}
	@keyframes glass-wobble {
		0%,
		100% {
			transform: rotate(-12deg);
		}
		50% {
			transform: rotate(12deg);
		}
	}
	@keyframes glass-pulse {
		0%,
		100% {
			transform: scale(1);
		}
		40% {
			transform: scale(0.86);
		}
		60% {
			transform: scale(1.05);
		}
	}
</style>
