<script lang="ts">
	// Flow / Chat — Ask → think → answer (ENb07) → Answer: a reply's prose as the pen sets it (body
	// 15, line height 1.6, text-2), from markdown the server rendered and sanitised. Paragraphs 12
	// apart; lists, quotes, tables and code in the design's quiet styles; code highlighted in the
	// page's theme (the server sends both themes as --shiki-light / --shiki-dark). The HTML is
	// trusted only because the server sanitises it (src/lib/server/markdown.ts): never hand this
	// anything else.
	type Props = { html: string; dimmed?: boolean };
	let { html, dimmed = false }: Props = $props();
</script>

<div class={['prose w-full min-w-0 text-body leading-relaxed text-text-2 transition-opacity', dimmed && 'opacity-40']}>
	<!-- eslint-disable-next-line svelte/no-at-html-tags -- sanitised on the server -->
	{@html html}
</div>

<style>
	.prose :global(:where(p, ul, ol, blockquote, pre, table, h1, h2, h3, h4, h5, h6, hr) + :where(p, ul, ol, blockquote, pre, table, h1, h2, h3, h4, h5, h6, hr)) {
		margin-top: var(--nm-space-md);
	}
	.prose :global(strong) {
		font-weight: 600;
		color: var(--nm-text);
	}
	.prose :global(a) {
		color: var(--nm-text);
		text-decoration: underline;
		text-decoration-color: var(--nm-border-strong);
		text-underline-offset: 2px;
		border-radius: 4px;
	}
	.prose :global(a:hover) {
		text-decoration-color: var(--nm-text-2);
	}
	.prose :global(:where(h1, h2, h3, h4, h5, h6)) {
		font-size: var(--nm-size-body);
		font-weight: 600;
		color: var(--nm-text);
	}
	.prose :global(:where(ul, ol)) {
		padding-left: 20px;
	}
	.prose :global(ul) {
		list-style: disc;
	}
	.prose :global(ol) {
		list-style: decimal;
	}
	.prose :global(li + li) {
		margin-top: var(--nm-space-xs);
	}
	.prose :global(li::marker) {
		color: var(--nm-text-3);
	}
	.prose :global(blockquote) {
		border-left: 1px solid var(--nm-border-strong);
		padding-left: 13px;
		color: var(--nm-text-3);
	}
	.prose :global(hr) {
		border-top: 1px solid var(--nm-border);
	}
	.prose :global(:not(pre) > code) {
		font-family: var(--nm-mono);
		font-size: 0.87em;
		padding: 1px 4px;
		border-radius: 4px;
		background: var(--nm-surface-2);
		color: var(--nm-text);
	}
	.prose :global(pre) {
		font-family: var(--nm-mono);
		font-size: var(--nm-size-caption);
		line-height: 1.7;
		padding: 12px 14px;
		border-radius: var(--nm-radius-md);
		outline: 1px solid var(--nm-border);
		outline-offset: -1px;
		background: var(--nm-surface) !important;
		overflow-x: auto;
	}
	/* Each token carries both themes' colours (--shiki-light, --shiki-dark): the page's theme picks. */
	.prose :global(pre.shiki span) {
		color: light-dark(var(--shiki-light), var(--shiki-dark));
	}
	.prose :global(table) {
		display: block;
		max-width: 100%;
		overflow-x: auto;
		border-collapse: collapse;
		font-size: var(--nm-size-ui);
	}
	.prose :global(:where(th, td)) {
		padding: 6px 12px;
		border-bottom: 1px solid var(--nm-border);
		text-align: left;
	}
	.prose :global(th) {
		font-family: var(--nm-mono);
		font-size: var(--nm-size-label);
		font-weight: 400;
		color: var(--nm-text-3);
		text-transform: uppercase;
		letter-spacing: 0.06em;
	}
</style>
