<script lang="ts">
	import { agentLook, GRADIENT_STOPS, shapePath, type GradientTone, type ShapeMotion, type ShapeName } from './shapes';

	let {
		agent,
		shape,
		tone,
		motion,
		size = 40,
		working = false,
		face = false,
		label,
		class: extraClass = '',
	}: {
		/** agent_type or display name — picks the shape + gradient (see agentLook). */
		agent?: string | null;
		shape?: ShapeName;
		tone?: GradientTone;
		/** How it moves while working; defaults to the agent's own (see agentLook). */
		motion?: ShapeMotion;
		size?: number;
		/** Moves (in the agent's motion) while the agent is actually doing something. */
		working?: boolean;
		/** The two-dot face: Nomi's brand mark, and every crew member in chat. */
		face?: boolean;
		/** When set the shape is announced (role="img"); otherwise it's decorative. */
		label?: string;
		class?: string;
	} = $props();

	const uid = $props.id();
	const look = $derived(agentLook(agent));
	const resolvedShape = $derived(shape ?? look.shape);
	const resolvedTone = $derived(tone ?? look.tone);
	const resolvedMotion = $derived(motion ?? look.motion);
	const stops = $derived(GRADIENT_STOPS[resolvedTone]);
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 48 48"
	class="agent-shape {extraClass}"
	class:agent-shape--working={working}
	data-motion={resolvedMotion}
	role={label ? 'img' : undefined}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
>
	<defs>
		<linearGradient id="{uid}-g" x1="0" y1="0" x2="1" y2="1">
			{#each stops as stop, i (i)}
				<stop offset={stops.length === 1 ? 0 : i / (stops.length - 1)} stop-color={stop} />
			{/each}
		</linearGradient>
	</defs>
	<path d={shapePath(resolvedShape)} fill="url(#{uid}-g)" />
	{#if face}
		<circle cx="19" cy="22" r="2.6" fill="var(--nomi-on-gradient-glow)" />
		<circle cx="29" cy="22" r="2.6" fill="var(--nomi-on-gradient-glow)" />
	{/if}
</svg>

<style>
	.agent-shape {
		flex: none;
		display: block;
		transform-origin: 50% 50%;
	}
	/* Each motion is a spatial animation on the expressive spring curve; colour never animates. */
	.agent-shape--working[data-motion='spin'] {
		animation:
			agent-shape-spin 2.6s linear infinite,
			agent-shape-breathe 1.7s cubic-bezier(0.38, 1.21, 0.22, 1) infinite;
	}
	.agent-shape--working[data-motion='wobble'] {
		animation: agent-shape-wobble 1.1s cubic-bezier(0.38, 1.21, 0.22, 1) infinite;
	}
	.agent-shape--working[data-motion='bounce'] {
		animation: agent-shape-bounce 0.9s cubic-bezier(0.38, 1.21, 0.22, 1) infinite;
	}
	.agent-shape--working[data-motion='pulse'] {
		animation: agent-shape-pulse 1.3s cubic-bezier(0.38, 1.21, 0.22, 1) infinite;
	}
	.agent-shape--working[data-motion='orbit'] {
		animation:
			agent-shape-spin 4.2s linear infinite,
			agent-shape-orbit 2.1s ease-in-out infinite;
	}
	@keyframes agent-shape-wobble {
		0%,
		100% {
			rotate: -14deg;
		}
		50% {
			rotate: 14deg;
		}
	}
	@keyframes agent-shape-bounce {
		0%,
		100% {
			translate: 0 0;
			scale: 1 1;
		}
		35% {
			translate: 0 -14%;
			scale: 0.94 1.06;
		}
		70% {
			translate: 0 0;
			scale: 1.08 0.9;
		}
	}
	@keyframes agent-shape-pulse {
		0%,
		100% {
			scale: 1;
		}
		40% {
			scale: 0.78;
		}
		60% {
			scale: 1.06;
		}
	}
	@keyframes agent-shape-orbit {
		0%,
		100% {
			translate: 6% 0;
		}
		25% {
			translate: 0 6%;
		}
		50% {
			translate: -6% 0;
		}
		75% {
			translate: 0 -6%;
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.agent-shape--working[data-motion] {
			animation: none;
		}
	}
	@keyframes agent-shape-spin {
		to {
			rotate: 360deg;
		}
	}
	@keyframes agent-shape-breathe {
		0%,
		100% {
			scale: 1;
		}
		50% {
			scale: 0.86;
		}
	}
</style>
