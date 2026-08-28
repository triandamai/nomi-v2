<script lang="ts">
	import type { RenderedMessage } from '$lib/types';

	let { message }: { message: RenderedMessage } = $props();
</script>

<div class="flex {message.sender === 'user' ? 'justify-end' : 'justify-start'}">
	<div
		class="message-bubble md-body-large max-w-md px-4 py-2"
		style={message.sender === 'user'
			? `background: var(--md-sys-color-primary-container); color: var(--md-sys-color-on-primary-container); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small) var(--md-sys-shape-corner-large)`
			: `background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)`}
	>
		{@html message.content_html}
	</div>
</div>

<style>
	/* content_html is server-rendered from trusted markdown+shiki output (sanitized before it
	   ever reaches this component — see $lib/server/markdown.ts), injected via {@html}. Svelte's
	   scoped-style attribute isn't applied to {@html} content, so every rule below needs :global. */
	.message-bubble :global(p) {
		margin: 0;
	}
	.message-bubble :global(p + p) {
		margin-top: 0.5em;
	}
	.message-bubble :global(pre) {
		overflow-x: auto;
		padding: 12px;
		border-radius: var(--md-sys-shape-corner-small);
		margin: 0.5em 0;
	}
	.message-bubble :global(pre.shiki-fallback) {
		background: var(--md-sys-color-surface-container-highest);
	}
	.message-bubble :global(code) {
		font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
		font-size: 0.875em;
	}
	.message-bubble :global(:not(pre) > code) {
		background: color-mix(in srgb, currentColor 12%, transparent);
		padding: 0.1em 0.3em;
		border-radius: var(--md-sys-shape-corner-extra-small);
	}
	.message-bubble :global(ul),
	.message-bubble :global(ol) {
		margin: 0.5em 0;
		padding-left: 1.5em;
	}
	.message-bubble :global(blockquote) {
		margin: 0.5em 0;
		padding-left: 0.75em;
		border-left: 3px solid currentColor;
		opacity: 0.85;
	}
	.message-bubble :global(a) {
		color: inherit;
		text-decoration: underline;
	}
	.message-bubble :global(table) {
		border-collapse: collapse;
		margin: 0.5em 0;
	}
	.message-bubble :global(th),
	.message-bubble :global(td) {
		border: 1px solid var(--md-sys-color-outline-variant);
		padding: 4px 8px;
	}

	/* Shiki's dual-theme output: literal light colors on the element, dark overrides available
	   as --shiki-dark/--shiki-dark-bg custom properties for us to apply under the same
	   prefers-color-scheme switch the rest of the app already themes with. */
	@media (prefers-color-scheme: dark) {
		.message-bubble :global(.shiki),
		.message-bubble :global(.shiki span) {
			color: var(--shiki-dark);
			background-color: var(--shiki-dark-bg);
		}
	}
</style>
