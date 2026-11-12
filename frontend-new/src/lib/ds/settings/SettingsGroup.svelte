<script lang="ts">
	import type { Snippet } from 'svelte';

	// App / Settings / General (m7lvY1) → Panel: one group of settings, its name (15/500) and what
	// it covers over its rows, 12 apart; the rows sit between strokes. `id` is its anchor, for the
	// page's On this page.
	type Props = { id: string; title: string; description?: string; children: Snippet };
	let { id, title, description, children }: Props = $props();
</script>

<section {id} aria-labelledby="{id}-title" class="flex scroll-mt-6 flex-col gap-3">
	<header class="flex flex-col gap-0.5">
		<h2 id="{id}-title" class="text-body font-medium text-text">{title}</h2>
		{#if description}<p class="text-caption text-text-3">{description}</p>{/if}
	</header>
	<div class="flex flex-col border-t border-border">{@render children()}</div>
</section>
