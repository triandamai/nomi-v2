<script lang="ts">
	import type { CrewId } from './crew';
	import { CREW_LOOKS, FACE_COLOR, FACE_EYES, SHAPE_VIEW, TONE_STOPS, shapePath } from './shapes';

	// A crew member's agent shape, as frontend draws it (m3/AgentShape, which frontend's twin of this
	// component renders): their own outline in their own gradient, with the two-dot face. This leaves
	// the pen's flat "Mascot / <name>" frames on purpose (AGENTS.md §9); they are kept in
	// $lib/rollback/PenMascot.svelte to roll back to.
	// Identity, not decoration: only where that agent speaks or works. Decorative next to their
	// name; give it a label when it stands alone.
	let { agent, size = 96, label }: { agent: CrewId; size?: 96 | 32 | 20 | 16; label?: string } = $props();

	const uid = $props.id();
	const look = $derived(CREW_LOOKS[agent]);
	const stops = $derived(TONE_STOPS[look.tone]);
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 {SHAPE_VIEW} {SHAPE_VIEW}"
	role={label ? 'img' : undefined}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
	data-shape={look.shape}
	class="shrink-0"
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
