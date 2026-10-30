<script lang="ts">
	/** A keyboard key or chord, e.g. ⌘K. */
	let { keys }: { keys: string } = $props();
</script>

<kbd class="inline-flex h-5 items-center rounded-xs border border-border-strong px-1.5 font-mono text-label text-text-2">{keys}</kbd>
