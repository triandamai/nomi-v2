<script lang="ts">
	/** A section's name in the mono meta face, and how many are in it. */
	let { label, count }: { label: string; count?: number | string } = $props();
</script>

<p class="section">
	<span class="nomi-meta">{label}</span>
	{#if count !== undefined}<span class="count">{count}</span>{/if}
</p>

<style>
	.section {
		display: flex;
		align-items: center;
		gap: 8px;
		margin: 0;
	}
	.count {
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
