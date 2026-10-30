<script lang="ts">
	/** What the keys do here, in a quiet mono line under a field: "↵ send   ⇧↵ new line". */
	let { hints }: { hints: { keys: string; action: string }[] } = $props();
</script>

<p class="hints">
	{#each hints as hint (hint.keys)}<span>{hint.keys} {hint.action}</span>{/each}
</p>

<style>
	.hints {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 16px;
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
