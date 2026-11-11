<script lang="ts">
	import type { Snippet } from 'svelte';
	import Status from '../status/Status.svelte';

	// App / Settings / Profile (U9Gnrg) → Showcase: a live project the person features, outlined in
	// border-strong, 10 in: its tile (88×56; the pen draws a screenshot of the app, which Nomi
	// doesn't keep, so this is the project's initial), its name, Live, how it was built in mono,
	// and its tools at the end.
	type Props = { name: string; status: string; meta?: string; href?: string; tools?: Snippet };
	let { name, status, meta, href, tools }: Props = $props();
</script>

<article aria-label={name} class="flex items-center gap-3 rounded-md p-2.5 outline -outline-offset-1 outline-border-strong">
	<span aria-hidden="true" class="flex h-14 w-22 shrink-0 items-center justify-center rounded-sm bg-surface-2 font-mono text-heading text-text-2">{name.charAt(0).toUpperCase()}</span>
	<div class="flex min-w-0 flex-1 flex-col gap-1">
		{#if href}
			<a {href} target="_blank" rel="noopener noreferrer" class="truncate rounded-xs text-ui font-medium text-text outline-offset-2 hover:underline">{name}</a>
		{:else}
			<span class="truncate text-ui font-medium text-text">{name}</span>
		{/if}
		<Status kind="live" label={status} />
		{#if meta}<span class="truncate font-mono text-label text-text-3">{meta}</span>{/if}
	</div>
	{#if tools}<div class="flex shrink-0 items-center">{@render tools()}</div>{/if}
</article>
