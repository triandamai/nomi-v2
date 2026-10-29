<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * A board inside a sheet section, as in the pen file: its name, a note in mono, the examples.
	 * The Build sheet puts the note right after the name (`inline`); the others right-align it.
	 */
	let { title, note, inline = false, children }: { title: string; note: string; inline?: boolean; children: Snippet } = $props();
</script>

<div data-board class="flex min-w-0 flex-col gap-5 rounded-md border border-border p-6">
	<p class={['flex items-baseline gap-3 text-ui font-medium text-text', !inline && 'justify-between']}>
		{title} <span class={['font-mono text-label text-text-3', !inline && 'text-right']}>{note}</span>
	</p>
	{@render children()}
</div>
