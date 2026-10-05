<script lang="ts">
	import { agentLook, GRADIENT_STOPS, shapePath, type GradientTone, type ShapeName } from './shapes';

	let {
		agent,
		shape,
		tone,
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
		size?: number;
		/** Spins and breathes while the agent is actually doing something. */
		working?: boolean;
		/** Nomi's two-dot face — only for the Nomi brand mark. */
		face?: boolean;
		/** When set the shape is announced (role="img"); otherwise it's decorative. */
		label?: string;
		class?: string;
	} = $props();

	const uid = $props.id();
	const look = $derived(agentLook(agent));
	const resolvedShape = $derived(shape ?? look.shape);
	const resolvedTone = $derived(tone ?? look.tone);
	const stops = $derived(GRADIENT_STOPS[resolvedTone]);
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 48 48"
	class="agent-shape {extraClass}"
	class:agent-shape--working={working}
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
	.agent-shape--working {
		animation:
			agent-shape-spin 2.6s linear infinite,
			agent-shape-breathe 1.7s cubic-bezier(0.38, 1.21, 0.22, 1) infinite;
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
