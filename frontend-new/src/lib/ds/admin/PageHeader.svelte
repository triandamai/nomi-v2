<script lang="ts">
	import type { Snippet } from 'svelte';
	import { ArrowLeft } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Admin → Page header. The pen draws no admin screens; this is built from the design system's
	// type (AGENTS.md §2): the page's name as the screen's title (display), one line on what it's for
	// (ui, text-2: the console keeps to four sizes a screen), its actions at the end, and on a page
	// inside another a quiet way back above the title.
	type Props = {
		title: string;
		intro?: string;
		/** A page inside another (a plan, a person): back to the list. */
		back?: { href: string; label: string };
		/** Machine detail beside the title, in mono: an id, a slug, a count. */
		meta?: string;
		actions?: Snippet;
	};
	let { title, intro, back, meta, actions }: Props = $props();
</script>

<header class="flex w-full flex-col gap-3">
	{#if back}
		<a href={back.href} class="inline-flex w-fit items-center gap-1.5 rounded-xs text-caption text-text-2 outline-offset-2 hover:text-text pointer-coarse:min-h-11">
			<span class="inline-flex text-text-3"><Icon icon={ArrowLeft} size={14} /></span>
			{back.label}
		</a>
	{/if}
	<div class="flex w-full flex-wrap items-end justify-between gap-x-6 gap-y-3">
		<div class="flex min-w-0 flex-col gap-1.5">
			<div class="flex min-w-0 flex-wrap items-baseline gap-x-3 gap-y-1">
				<h1 class="text-display font-medium break-words text-text">{title}</h1>
				{#if meta}<span class="font-mono text-label break-all text-text-3">{meta}</span>{/if}
			</div>
			{#if intro}<p class="max-w-160 text-ui leading-normal text-text-2">{intro}</p>{/if}
		</div>
		{#if actions}<div class="flex shrink-0 flex-wrap items-center gap-2">{@render actions()}</div>{/if}
	</div>
</header>
