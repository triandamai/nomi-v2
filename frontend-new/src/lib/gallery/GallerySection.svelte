<script lang="ts">
	import type { Snippet } from 'svelte';

	/** One section of a design sheet, as laid out in the pen file: title, one line, then the content. */
	let { title, description, children }: { title: string; description: string; children: Snippet } = $props();
</script>

<section class="flex flex-col gap-6 border-t border-border py-12" aria-label={title}>
	<header class="flex flex-col gap-1">
		<h2 class="text-body font-medium text-text">{title}</h2>
		<p class="max-w-3xl text-caption text-text-2">{description}</p>
	</header>
	{@render children()}
</section>
