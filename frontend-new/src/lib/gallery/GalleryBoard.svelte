<script lang="ts">
	import type { Snippet } from 'svelte';

	/** A board inside a sheet section, as in the pen file: its name, a note in mono, the examples. */
	let { title, note, children }: { title: string; note: string; children: Snippet } = $props();
</script>

<div class="flex min-w-0 flex-col gap-4 rounded-md border border-border p-5">
	<p class="flex items-baseline justify-between gap-3 text-ui font-medium text-text">{title} <span class="text-right font-mono text-label text-text-3">{note}</span></p>
	{@render children()}
</div>
