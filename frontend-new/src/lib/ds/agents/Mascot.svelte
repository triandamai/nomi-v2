<script lang="ts">
	import { untrack } from 'svelte';
	import type { AvatarMotion } from 'open-bot-avatar';
	import type { MountedAvatar } from 'open-bot-avatar/dom';
	import { CREW, type CrewId } from './crew';
	import { CREW_LOOKS, FACE_COLOR, FACE_EYES, SHAPE_VIEW, TONE_BODY, TONE_STOPS, liveDefinition, shapePath } from './shapes';

	// A crew member's agent shape, as frontend draws it (m3/AgentShape, which frontend's twin of this
	// component renders): their own outline in their own gradient, with the two-dot face. This leaves
	// the pen's flat "Mascot / <name>" frames on purpose (AGENTS.md §9); they are kept in
	// $lib/rollback/PenMascot.svelte to roll back to.
	// Identity, not decoration: only where that agent speaks or works. Decorative next to their
	// name; give it a label when it stands alone.
	//
	// `live` (on by default at 96 and 32) brings the same outline to life with open-bot-avatar: it
	// gains volume, blinks, follows the pointer with its eyes and turns toward it, and plays `mood`
	// (idle, thinking while it works, happy). Hovering it cheers it up. The flat SVG is drawn first
	// (server render, no JS, before the engine mounts), so nothing jumps. At 20 and 16 it stays flat:
	// the eyes would be too small to read and a list can hold dozens. Reduced motion is followed.
	let {
		agent,
		size = 96,
		label,
		live = size >= 32,
		mood = 'idle',
	}: { agent: CrewId; size?: 96 | 32 | 20 | 16; label?: string; live?: boolean; mood?: AvatarMotion } = $props();

	const uid = $props.id();
	const look = $derived(CREW_LOOKS[agent]);
	const stops = $derived(TONE_STOPS[look.tone]);
	const name = $derived(CREW.find((member) => member.id === agent)?.name ?? agent);

	let host = $state<HTMLSpanElement>();
	let avatar = $state<MountedAvatar>();
	let hovered = $state(false);
	const shownMood = $derived<AvatarMotion>(hovered && mood === 'idle' ? 'happy' : mood);

	// Mount once per member (the engine is loaded only when a live mascot appears).
	$effect(() => {
		if (!live || !host) return;
		const el = host;
		const definition = liveDefinition(agent, name);
		const color = TONE_BODY[look.tone];
		let mounted: MountedAvatar | undefined;
		let gone = false;
		import('open-bot-avatar/dom').then(({ mountAvatar }) => {
			if (gone) return;
			mounted = mountAvatar(el, {
				definition,
				color,
				eyeColor: FACE_COLOR,
				followPointer: true,
				mood: untrack(() => shownMood),
				// The outer element carries the name (or hides it); the engine's own node stays quiet.
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
		avatar?.update({ mood: shownMood });
	});
</script>

{#snippet flat(extra?: string)}
	<svg
		width={size}
		height={size}
		viewBox="0 0 {SHAPE_VIEW} {SHAPE_VIEW}"
		role={label ? 'img' : undefined}
		aria-label={label}
		aria-hidden={label ? undefined : 'true'}
		data-shape={look.shape}
		class="shrink-0 {extra ?? ''}"
	>
		<defs>
			<linearGradient id="{uid}-paint" x1="0" y1="0" x2="1" y2="1">
				{#each stops as stop, index (index)}
					<stop offset={stops.length === 1 ? 0 : index / (stops.length - 1)} stop-color={stop} />
				{/each}
			</linearGradient>
		</defs>
		<path data-part="body" d={shapePath(look.shape)} fill="url(#{uid}-paint)" />
		{#each FACE_EYES as eye, index (index)}
			<circle data-part="eye" cx={eye.cx} cy={eye.cy} r={eye.r} fill={FACE_COLOR} />
		{/each}
	</svg>
{/snippet}

{#if live}
	<span
		class="relative inline-block shrink-0"
		style:width="{size}px"
		style:height="{size}px"
		aria-hidden={label ? undefined : 'true'}
		data-shape={look.shape}
		data-live={avatar ? 'mounted' : 'pending'}
		data-mood={shownMood}
		onpointerenter={() => (hovered = true)}
		onpointerleave={() => (hovered = false)}
	>
		{@render flat(avatar ? 'opacity-0' : '')}
		<span bind:this={host} class="absolute inset-0" aria-hidden="true"></span>
	</span>
{:else}
	{@render flat()}
{/if}
