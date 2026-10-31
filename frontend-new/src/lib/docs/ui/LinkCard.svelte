<script lang="ts">
	import type { Snippet } from 'svelte';

	// A way into a part of the docs: its name and one line on what's there.
	type Props = { href: string; title: string; blurb?: string; eyebrow?: string; visual?: Snippet };
	let { href, title, blurb, eyebrow, visual }: Props = $props();
</script>

<a {href} class="group flex flex-col gap-1.5 rounded-md border border-border px-4 pt-3.5 pb-4 transition-colors hover:border-border-strong hover:bg-surface">
	{#if visual}<span class="mb-2 flex">{@render visual()}</span>{/if}
	{#if eyebrow}<span class="font-mono text-label text-text-3">{eyebrow}</span>{/if}
	<span class="text-ui font-medium text-text">{title}</span>
	{#if blurb}<span class="text-caption text-text-3">{blurb}</span>{/if}
</a>
