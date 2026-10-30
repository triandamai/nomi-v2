<script lang="ts">
	/** What the keys do here, in a quiet mono line under a field: "↵ send   ⇧↵ new line". */
	let { hints }: { hints: { keys: string; action: string }[] } = $props();
</script>

<p class="flex flex-wrap gap-x-4 gap-y-1 font-mono text-label text-text-3">
	{#each hints as hint (hint.keys)}<span>{hint.keys} {hint.action}</span>{/each}
</p>
