<script lang="ts">
	import type { CrewId } from '$lib/ds/agents/crew';
	import { MASCOTS } from '$lib/ds/agents/mascots';

	// The pen's own mascots (Nomi.pen, "Mascot / <name>"), kept as they were before Mascot took
	// frontend's agent shapes: to roll back, copy this over ds/agents/Mascot.svelte.
	// Identity, not decoration: only where that agent speaks or works. Decorative next to their
	// name; give it a label when it stands alone.
	let { agent, size = 96, label }: { agent: CrewId; size?: 96 | 32 | 20 | 16; label?: string } = $props();

	const shape = $derived(MASCOTS[agent]);
</script>

<svg
	width={size}
	height={size}
	viewBox="0 0 96 96"
	role={label ? 'img' : undefined}
	aria-label={label}
	aria-hidden={label ? undefined : 'true'}
	class="shrink-0"
>
	{#if shape.body.kind === 'path'}
		<path data-part="body" transform="translate({shape.body.x} {shape.body.y})" d={shape.body.d} style="fill: var(--crew-{agent})" />
	{:else}
		<circle data-part="body" cx={shape.body.cx} cy={shape.body.cy} r={shape.body.r} style="fill: var(--crew-{agent})" />
	{/if}
	{#each shape.eyes as eye, index (index)}
		<circle data-part="eye" cx={eye.cx} cy={eye.cy} r={eye.r} fill={shape.eyeColor} />
	{/each}
</svg>
