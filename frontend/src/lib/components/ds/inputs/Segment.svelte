<script lang="ts">
	import { Check } from '@lucide/svelte';

	// One view in a Segmented control, as an M3 segment: the chosen one filled in
	// secondary-container with a check, with an optional count. Presentational.
	type Props = { label: string; count?: number | string; checked?: boolean };
	let { label, count, checked = false }: Props = $props();
</script>

<span class="segment" class:segment--checked={checked}>
	{#if checked}<Check size={18} aria-hidden="true" />{/if}
	<span>{label}</span>
	{#if count !== undefined}<span class="count">{count}</span>{/if}
</span>

<style>
	.segment {
		display: inline-flex;
		height: 40px;
		align-items: center;
		gap: 8px;
		padding: 0 16px;
		color: var(--md-sys-color-on-surface);
		font: var(--md-sys-typescale-label-large-weight) var(--md-sys-typescale-label-large-size) / var(--md-sys-typescale-label-large-line-height) var(--md-sys-typescale-label-large-font);
	}
	.segment--checked {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.count {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		opacity: 0.72;
	}
</style>
