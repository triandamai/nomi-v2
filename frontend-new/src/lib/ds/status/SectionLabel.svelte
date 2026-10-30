<script lang="ts">
	/** The only uppercase style: a section's name in mono capitals, and how many are in it. */
	let { label, count }: { label: string; count?: number | string } = $props();
</script>

<p class="flex items-center gap-2 font-mono text-label text-text-3">
	<span class="tracking-label uppercase">{label}</span>
	{#if count !== undefined}<span>{count}</span>{/if}
</p>
