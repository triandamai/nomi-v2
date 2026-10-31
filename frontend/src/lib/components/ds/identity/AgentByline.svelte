<script lang="ts">
	import Mascot from '../agents/Mascot.svelte';
	import type { CrewId } from '../agents/crew';

	// Who is speaking, and when (in mono: "15:37 · 4m 12s").
	let { agent, name, meta }: { agent: CrewId; name: string; meta?: string } = $props();
</script>

<span class="byline">
	<Mascot {agent} size={20} />
	<span class="name">{name}</span>
	{#if meta}<span class="meta">{meta}</span>{/if}
</span>

<style>
	.byline {
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}
	.name {
		font: var(--md-sys-typescale-title-small-weight) var(--md-sys-typescale-title-small-size) / var(--md-sys-typescale-title-small-line-height) var(--md-sys-typescale-title-small-font);
		color: var(--md-sys-color-on-surface);
	}
	.meta {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
