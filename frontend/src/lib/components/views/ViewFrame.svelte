<!-- The surface every view sits on, with its optional title: one place for the views' shared
     look (corner, padding, ground), so a new view only draws its own content. -->
<script lang="ts">
	import type { Snippet } from 'svelte';

	let {
		title,
		flush = false,
		children,
	}: {
		title?: string | null;
		/** Content runs to the frame's edges (a table, a terminal). */
		flush?: boolean;
		children: Snippet;
	} = $props();
</script>

<section class="view" class:view--flush={flush}>
	{#if title}<h3 class="view__title">{title}</h3>{/if}
	{@render children()}
</section>

<style>
	.view {
		display: flex;
		flex-direction: column;
		gap: 12px;
		min-width: 0;
		padding: 16px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		box-shadow: inset 0 0 0 1px var(--md-sys-color-outline-variant);
	}
	.view--flush {
		padding: 0;
		overflow: hidden;
	}
	.view--flush .view__title {
		padding: 14px 16px 0;
	}
	.view__title {
		margin: 0;
		font-size: 1rem;
		font-weight: 700;
		line-height: 1.35;
	}
</style>
