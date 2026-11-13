<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / Crew (vVUeh) → Group: a group of the roster, its mono label with a note on
	// the right ("always on", how many), then its members 2 apart.
	type Props = { label: string; note?: string; children: Snippet };
	let { label, note, children }: Props = $props();

	const id = $props.id();
</script>

<section aria-labelledby={id} class="flex flex-col gap-0.5">
	<h3 class="flex gap-2 px-2 pb-1 font-mono text-label text-text-3">
		<span {id} class="min-w-0 flex-1 tracking-label uppercase">{label}</span>
		{#if note}<span>{note}</span>{/if}
	</h3>
	<ul class="flex flex-col gap-0.5">{@render children()}</ul>
</section>
