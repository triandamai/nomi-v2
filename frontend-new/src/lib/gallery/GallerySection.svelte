<script lang="ts">
	import type { Snippet } from 'svelte';
	import { slugify } from '$lib/docs/slug';

	/**
	 * One section of a design sheet, as the pen lays it out: its Section Head (a 28px title with the
	 * line beside it), then the boards. The sheet puts 64px between sections. Its id is its title, so
	 * the docs can link to it. Its name is the pen's section frame's (`label`), where that differs
	 * from the title it shows ("Section / Projects" titled "Cards & overlays"): the pen tools and
	 * the parity guard find sections by it.
	 */
	let { title, label, description, children }: { title: string; label?: string; description: string; children: Snippet } = $props();
</script>

<section id={slugify(title)} class="flex flex-col gap-6" aria-label={label ?? title}>
	<header class="flex flex-wrap items-end gap-x-6 gap-y-1">
		<h2 class="text-display font-medium tracking-display text-text">{title}</h2>
		<p class="min-w-0 flex-1 basis-80 text-ui text-text-3">{description}</p>
	</header>
	{@render children()}
</section>
