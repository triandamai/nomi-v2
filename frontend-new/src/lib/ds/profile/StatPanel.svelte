<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / Profile (U9Gnrg) → Panel: one of the breakdowns under the heatmap (Models you
	// use most, Your crew), outlined, 16 in: its name, a mono note at the end, then what it holds,
	// 12 apart.
	type Props = { title: string; meta?: string; children: Snippet };
	let { title, meta, children }: Props = $props();

	const id = $props.id();
</script>

<section aria-labelledby={id} class="flex min-w-0 flex-col gap-3 rounded-md p-4 outline -outline-offset-1 outline-border">
	<header class="flex items-center justify-between gap-2">
		<h2 {id} class="text-ui font-medium text-text">{title}</h2>
		{#if meta}<span class="font-mono text-label text-text-3">{meta}</span>{/if}
	</header>
	{@render children()}
</section>
