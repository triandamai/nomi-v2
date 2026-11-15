<script lang="ts">
	import { untrack } from 'svelte';
	import type { AvatarMotion } from 'open-bot-avatar';
	import type { MountedAvatar } from 'open-bot-avatar/dom';
	import {
		agentLook,
		GRADIENT_STOPS,
		liveDefinition,
		shapePath,
		TONE_BODY,
		type GradientTone,
		type ShapeMotion,
		type ShapeName,
	} from './shapes';

	let {
		agent,
		shape,
		tone,
		motion,
		size = 40,
		working = false,
		face = false,
		label,
		paint,
		live = face && size >= 32,
		mood,
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
		/** Gradient stops (any CSS colour, var() included) in place of the tone's own. */
		paint?: string[];
		/** When face is shown and size >= 32, renders an interactive open-bot-avatar character. */
		live?: boolean;
		/** Overrides avatar mood ('idle' | 'thinking' | 'happy'). Defaults to thinking when working, else idle. */
		mood?: AvatarMotion;
		class?: string;
	} = $props();

	const uid = $props.id();
	const look = $derived(agentLook(agent));
	const resolvedShape = $derived(shape ?? look.shape);
	const resolvedTone = $derived(tone ?? look.tone);
	const resolvedMotion = $derived(motion ?? look.motion);
	const stops = $derived(paint ?? GRADIENT_STOPS[resolvedTone]);

	let host = $state<HTMLSpanElement>();
	let avatar = $state<MountedAvatar>();
	let hovered = $state(false);

	const effectiveMood = $derived<AvatarMotion>(
		mood ?? (working ? 'thinking' : hovered ? 'happy' : 'idle'),
	);

	$effect(() => {
		if (!live || !host) return;
		const el = host;
		const definition = liveDefinition(resolvedShape, agent ?? resolvedShape);
		const color = TONE_BODY[resolvedTone];
		let mounted: MountedAvatar | undefined;
		let gone = false;

		import('open-bot-avatar/dom').then(({ mountAvatar }) => {
			if (gone) return;
			mounted = mountAvatar(el, {
				definition,
				color,
				eyeColor: '#062019',
				followPointer: true,
				mood: untrack(() => effectiveMood),
				label: '',
			});
			avatar = mounted;
		});

		return () => {
			gone = true;
			mounted?.destroy();
			avatar = undefined;
		};
	});

	$effect(() => {
		avatar?.update({ mood: effectiveMood });
	});
</script>

{#snippet flat(extra?: string)}
	<svg
		width={size}
		height={size}
		viewBox="0 0 48 48"
		class="agent-shape {extraClass} {extra ?? ''}"
		class:agent-shape--working={working}
		data-motion={resolvedMotion}
		role={label ? 'img' : undefined}
		aria-label={label}
		aria-hidden={label ? undefined : 'true'}
	>
		<defs>
			<linearGradient id="{uid}-g" x1="0" y1="0" x2="1" y2="1">
				{#each stops as stop, i (i)}
					<stop offset={stops.length === 1 ? 0 : i / (stops.length - 1)} style="stop-color: {stop}" />
				{/each}
			</linearGradient>
		</defs>
		<path d={shapePath(resolvedShape)} fill="url(#{uid}-g)" />
		{#if face}
			<circle cx="19" cy="22" r="2.6" fill="var(--nomi-on-gradient-glow)" />
			<circle cx="29" cy="22" r="2.6" fill="var(--nomi-on-gradient-glow)" />
		{/if}
	</svg>
{/snippet}

{#if live}
	<span
		class="agent-shape-host relative inline-block shrink-0 {extraClass}"
		style:width="{size}px"
		style:height="{size}px"
		aria-hidden={label ? undefined : 'true'}
		data-shape={resolvedShape}
		data-live={avatar ? 'mounted' : 'pending'}
		data-mood={effectiveMood}
		onpointerenter={() => (hovered = true)}
		onpointerleave={() => (hovered = false)}
	>
		{@render flat(avatar ? 'opacity-0' : '')}
		<span bind:this={host} class="absolute inset-0" aria-hidden="true"></span>
	</span>
{:else}
	{@render flat()}
{/if}

<style>
	.agent-shape {
		flex: none;
		display: block;
		transform-origin: 50% 50%;
	}
	.agent-shape-host {
		display: inline-block;
		vertical-align: middle;
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
