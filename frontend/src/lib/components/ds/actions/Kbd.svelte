<script lang="ts">
	/** A keyboard key or chord, e.g. ⌘K. */
	let { keys }: { keys: string } = $props();
</script>

<kbd class="kbd">{keys}</kbd>

<style>
	.kbd {
		display: inline-flex;
		align-items: center;
		height: 20px;
		padding: 0 6px;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: var(--md-sys-color-surface-container-high);
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
