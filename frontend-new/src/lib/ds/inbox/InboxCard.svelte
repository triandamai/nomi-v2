<script lang="ts">
	import type { Snippet } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import SourceTile from './SourceTile.svelte';
	import type { CrewId } from '../agents/crew';

	// App / Inbox (BFoiZ) → Needs You → Item: something waiting on the person, as a card (surface,
	// border-strong drawn inside, [14,16]): when in mono, whose tile, the title with its mono tag,
	// what it's about, then its actions 6 below with a quiet note after them; unread wears the dot.
	type Props = {
		time: string;
		agent: CrewId;
		title: string;
		tag?: string;
		caption?: string;
		unread?: boolean;
		/** A quiet mono note after the actions ("expires in 4 days"). */
		note?: string;
		actions?: Snippet;
	};
	let { time, agent, title, tag, caption, unread = false, note, actions }: Props = $props();

	const id = $props.id();
</script>

<article aria-labelledby="{id}-title" class="flex w-full gap-3 rounded-md bg-surface px-4 py-3.5 outline -outline-offset-1 outline-border-strong">
	<span class="w-14 shrink-0 font-mono text-label text-text-3">{time}</span>
	<SourceTile {agent} />
	<div class="flex min-w-0 flex-1 flex-col gap-0.75">
		<p class="flex min-w-0 items-center gap-2">
			<span id="{id}-title" class="min-w-0 text-ui font-medium text-text">{title}</span>
			{#if tag}<span class="shrink-0 font-mono text-label text-text-3">{tag}</span>{/if}
		</p>
		{#if caption}<p class="text-caption text-text-3">{caption}</p>{/if}
		{#if actions || note}
			<div class="flex flex-wrap items-center gap-2 pt-1.5">
				{@render actions?.()}
				{#if note}<span class="font-mono text-label text-text-3">{note}</span>{/if}
			</div>
		{/if}
	</div>
	{#if unread}
		<span data-unread class="size-1.5 shrink-0 rounded-full bg-accent"><span class="sr-only">{m.inbox_unread()}</span></span>
	{/if}
</article>
