<script lang="ts">
	// How much of something is used, as a bar and in words ("21.8K / 1M tokens this month").
	// The real figure, never more than full. `roomy` spaces it out, as in the drawer's usage card.
	type Props = { label: string; value: number; caption?: string; roomy?: boolean };
	let { label, value, caption, roomy = false }: Props = $props();

	const percent = $derived(Math.round(Math.min(Math.max(value, 0), 1) * 100));
	const id = $props.id();
</script>

<div
	role="meter"
	aria-labelledby="{id}-label"
	aria-valuemin={0}
	aria-valuemax={100}
	aria-valuenow={percent}
	aria-valuetext={caption ? `${percent}% · ${caption}` : `${percent}%`}
	class="meter"
	class:meter--roomy={roomy}
>
	<div class="head">
		<span id="{id}-label" class="label">{label}</span>
		<span class="meta">{percent}%</span>
	</div>
	<div data-track class="track">
		<div data-fill class="fill" style="width: {Math.min(Math.max(value, 0), 1) * 100}%"></div>
	</div>
	{#if caption}<span class="meta">{caption}</span>{/if}
</div>

<style>
	.meter {
		display: flex;
		width: 100%;
		flex-direction: column;
		gap: 6px;
	}
	.meter--roomy {
		gap: 10px;
	}
	.head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
	}
	.label {
		font: var(--md-sys-typescale-title-small-weight) var(--md-sys-typescale-title-small-size) / var(--md-sys-typescale-title-small-line-height) var(--md-sys-typescale-title-small-font);
		color: var(--md-sys-color-on-surface);
	}
	.meta {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.track {
		height: 4px;
		width: 100%;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
	}
	.fill {
		height: 100%;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary);
	}
</style>
