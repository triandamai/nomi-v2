<script lang="ts">
	import type { Snippet } from 'svelte';

	// Flow / Mobile — Home, Inbox & You, step 2 (fpYK0) → Needs you cards: one thing that waits on
	// the person, in a card (surface, border, radius 16, inset 14, 8 apart): its kind and when in mono,
	// the title at 16/500, what it's about at 14, and the actions the app can take (pills, 36 high).
	type Props = { kind: string; when: string; title: string; body?: string; unread?: boolean; actions?: Snippet; meta?: string };
	let { kind, when, title, body, unread = false, actions, meta }: Props = $props();
</script>

<article aria-label={title} class="flex flex-col gap-2 rounded-2xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
	<div class="flex items-center gap-2">
		{#if unread}<span aria-hidden="true" class="size-1.5 shrink-0 rounded-full bg-accent"></span>{/if}
		<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{kind}</span>
		<span class="shrink-0 font-mono text-label text-text-3">{when}</span>
	</div>
	<h3 class="text-phone font-medium text-text">{title}</h3>
	{#if body}<p class="text-phone-sm leading-cozy text-text-2">{body}</p>{/if}
	{#if actions || meta}
		<div class="flex flex-wrap items-center gap-2 pt-1">
			{@render actions?.()}
			{#if meta}<span class="ml-auto font-mono text-label text-text-3">{meta}</span>{/if}
		</div>
	{/if}
</article>
