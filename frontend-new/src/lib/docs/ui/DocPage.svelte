<script lang="ts">
	import type { Snippet } from 'svelte';
	import SectionLabel from '$lib/ds/status/SectionLabel.svelte';

	// A docs page: where it sits in the docs, its title, one line on what it's for, then the page.
	// Text pages keep an 800 measure; `wide` makes room for examples and tables.
	type Props = { eyebrow: string; title: string; lede?: string; wide?: boolean; meta?: Snippet; children: Snippet };
	let { eyebrow, title, lede, wide = false, meta, children }: Props = $props();
</script>

<svelte:head><title>{title} · Nomi design system</title></svelte:head>

<article class={['flex w-full flex-col gap-10', wide ? 'max-w-300' : 'max-w-200']}>
	<header class="flex flex-col gap-3 border-b border-border pb-8">
		<SectionLabel label={eyebrow} />
		<h1 class="text-display font-medium tracking-display text-text">{title}</h1>
		{#if lede}<p class="max-w-170 text-body leading-relaxed text-text-2">{lede}</p>{/if}
		{#if meta}<div class="flex flex-wrap items-center gap-2 pt-1">{@render meta()}</div>{/if}
	</header>
	<div class="doc-prose flex flex-col gap-12">{@render children()}</div>
</article>

<style>
	/* Running text inside a docs page, so pages can be written as plain HTML. */
	.doc-prose :global(p:not([class])) {
		margin: 0;
		max-width: 680px;
		font-size: var(--nm-size-body);
		line-height: 24px;
		color: var(--nm-text-2);
	}
	.doc-prose :global(:is(p, li) > strong) {
		font-weight: 500;
		color: var(--nm-text);
	}
	.doc-prose :global(:is(p, li, td) > a:not([class])) {
		color: var(--nm-text);
		text-decoration: underline;
		text-decoration-color: var(--nm-border-strong);
		text-underline-offset: 3px;
	}
	.doc-prose :global(:is(p, li, td) > a:not([class]):hover) {
		text-decoration-color: var(--nm-text);
	}
	.doc-prose :global(:is(p, li, td, h3) > code) {
		padding: 1px 4px;
		border-radius: 4px;
		background: var(--nm-surface-2);
		font-family: var(--nm-mono);
		font-size: 0.87em;
		color: var(--nm-text);
	}
	.doc-prose :global(ul:not([class])),
	.doc-prose :global(ol:not([class])) {
		display: flex;
		flex-direction: column;
		gap: 6px;
		max-width: 680px;
		margin: 0;
		padding-left: 20px;
		font-size: var(--nm-size-body);
		line-height: 24px;
		color: var(--nm-text-2);
	}
	.doc-prose :global(ul:not([class])) {
		list-style: disc;
	}
	.doc-prose :global(ol:not([class])) {
		list-style: decimal;
	}
	.doc-prose :global(:is(ul, ol):not([class]) > li::marker) {
		color: var(--nm-text-3);
	}
	.doc-prose :global(h3:not([class])) {
		margin: 8px 0 0;
		font-size: var(--nm-size-body);
		line-height: 19px;
		font-weight: 500;
		color: var(--nm-text);
	}
</style>
