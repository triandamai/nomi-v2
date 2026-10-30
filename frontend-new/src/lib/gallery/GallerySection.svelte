<script lang="ts">
	import type { Snippet } from 'svelte';

	/**
	 * One section of a design sheet, as the pen lays it out: its Section Head (a 28px title with the
	 * line beside it), then the boards; 64px between sections.
	 */
	let { title, description, children }: { title: string; description: string; children: Snippet } = $props();
</script>

<section class="flex flex-col gap-6 py-8" aria-label={title}>
	<header class="flex flex-wrap items-end gap-x-6 gap-y-1">
		<h2 class="text-display font-medium tracking-display text-text">{title}</h2>
		<p class="min-w-0 flex-1 basis-80 text-ui text-text-3">{description}</p>
	</header>
	{@render children()}
</section>
