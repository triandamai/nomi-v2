<script lang="ts">
	import { m } from '$lib/paraglide/messages';

	// Chat responses → Citation: the number of a source, right after the claim it backs. Screen
	// readers hear which source it is.
	type Props = { n: number; title: string; href?: string };
	let { n, title, href }: Props = $props();
</script>

{#if href}
	<a
		{href}
		aria-label={m.cite_label({ n: String(n), title })}
		class="inline-flex h-4 items-center rounded-xs bg-surface-2 px-1.25 align-middle font-mono text-micro text-text-2 no-underline outline-offset-1 hover:bg-border hover:text-text">{n}</a
	>
{:else}
	<span class="inline-flex h-4 items-center rounded-xs bg-surface-2 px-1.25 align-middle font-mono text-micro text-text-2">{n}</span>
{/if}
