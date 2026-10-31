<script lang="ts">
	import type { Snippet } from 'svelte';
	import { slugify } from '../slug';

	// A titled part of a docs page that can be linked to (#its-title).
	type Props = { title: string; id?: string; lede?: string; children: Snippet };
	let { title, id = slugify(title), lede, children }: Props = $props();
</script>

<section {id} class="flex scroll-mt-20 flex-col gap-4">
	<div class="flex flex-col gap-1.5">
		<h2 class="text-heading font-medium text-text"><a href="#{id}" class="hover:underline hover:decoration-border-strong hover:underline-offset-4">{title}</a></h2>
		{#if lede}<p class="max-w-170 text-ui leading-normal text-text-3">{lede}</p>{/if}
	</div>
	{@render children()}
</section>
