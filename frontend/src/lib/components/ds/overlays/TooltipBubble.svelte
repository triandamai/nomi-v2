<script lang="ts">
	// The tip itself, with the shortcut. Presentational: Tooltip shows it on hover or focus.
	type Props = { label: string; shortcut?: string; id?: string; element?: HTMLElement; [attribute: string]: unknown };
	let { label, shortcut, id, element = $bindable(), ...rest }: Props = $props();
</script>

<span bind:this={element} {id} role="tooltip" {...rest} class="tip {rest.class ?? ''}">
	{label}
	{#if shortcut}<span class="shortcut">{shortcut}</span>{/if}
</span>

<style>
	/* The M3 plain tooltip. */
	.tip {
		display: inline-flex;
		min-height: 24px;
		align-items: center;
		gap: 8px;
		padding: 4px 8px;
		border: 0;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
		pointer-events: none;
		font: var(--md-sys-typescale-body-small-weight) var(--md-sys-typescale-body-small-size) / var(--md-sys-typescale-body-small-line-height) var(--md-sys-typescale-body-small-font);
	}
	.shortcut {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		opacity: 0.72;
	}
</style>
