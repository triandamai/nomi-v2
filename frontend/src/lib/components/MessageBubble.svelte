<script lang="ts">
	import { deserialize } from '$app/forms';
	import IconCheck from './icons/IconCheck.svelte';
	import IconCopy from './icons/IconCopy.svelte';
	import IconShare from './icons/IconShare.svelte';
	import IconThumbDown from './icons/IconThumbDown.svelte';
	import IconThumbUp from './icons/IconThumbUp.svelte';
	import type { RenderedMessage } from '$lib/types';

	let { message }: { message: RenderedMessage } = $props();

	let bubbleEl: HTMLDivElement | undefined = $state();
	let feedback = $state(message.my_feedback);
	let messageCopied = $state(false);
	let shareCopied = $state(false);

	const senderLabel = message.sender === 'user' ? 'You' : 'Nomi';
	const formattedTime = new Date(message.created_at).toLocaleString(undefined, {
		dateStyle: 'medium',
		timeStyle: 'short',
	});

	// Hardcoded (not the $lib/components/icons/* components) because these get assigned via
	// innerHTML to plain DOM buttons built in enhanceCodeBlocks below — that code runs against
	// {@html}-injected markup, outside Svelte's own rendering, so it can't render a Svelte
	// component into it.
	const COPY_SVG =
		'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="9" y="9" width="11" height="11" rx="1.5"/><path d="M5 15V6a2 2 0 0 1 2-2h9"/></svg>';
	const CHECK_SVG =
		'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="5 12.5 10 17.5 19 6.5"/></svg>';
	const CHEVRON_UP_SVG =
		'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 15 12 9 18 15"/></svg>';
	const CHEVRON_DOWN_SVG =
		'<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 12 15 18 9"/></svg>';

	// Wraps every syntax-highlighted (or fallback) code block in a header (language + copy +
	// collapse) via real DOM APIs — never innerHTML of anything derived from message content, only
	// of the fixed icon constants above — so nothing about a chat message's own text can forge a
	// working button. Locates blocks via the `data-lang` attribute markdown.ts stamps on every
	// <pre>, both the shiki-highlighted and the unknown-language fallback path.
	function enhanceCodeBlocks(root: HTMLElement) {
		const blocks = root.querySelectorAll<HTMLElement>('pre[data-lang]');
		blocks.forEach((pre) => {
			if (pre.closest('.code-block')) return;

			const lang = pre.dataset.lang || 'text';

			const wrapper = document.createElement('div');
			wrapper.className = 'code-block';

			const header = document.createElement('div');
			header.className = 'code-block__header';

			const langLabel = document.createElement('span');
			langLabel.className = 'code-block__lang';
			langLabel.textContent = lang;

			const actions = document.createElement('div');
			actions.className = 'code-block__actions';

			const copyBtn = document.createElement('button');
			copyBtn.type = 'button';
			copyBtn.className = 'code-block__btn';
			copyBtn.setAttribute('aria-label', 'Copy code');
			copyBtn.innerHTML = COPY_SVG;
			copyBtn.addEventListener('click', () => {
				navigator.clipboard.writeText(pre.textContent ?? '');
				copyBtn.innerHTML = CHECK_SVG;
				copyBtn.setAttribute('aria-label', 'Copied');
				setTimeout(() => {
					copyBtn.innerHTML = COPY_SVG;
					copyBtn.setAttribute('aria-label', 'Copy code');
				}, 1500);
			});

			const collapseBtn = document.createElement('button');
			collapseBtn.type = 'button';
			collapseBtn.className = 'code-block__btn';
			collapseBtn.setAttribute('aria-label', 'Collapse code');
			collapseBtn.setAttribute('aria-expanded', 'true');
			collapseBtn.innerHTML = CHEVRON_UP_SVG;
			collapseBtn.addEventListener('click', () => {
				const collapsed = wrapper.classList.toggle('code-block--collapsed');
				collapseBtn.setAttribute('aria-expanded', String(!collapsed));
				collapseBtn.setAttribute('aria-label', collapsed ? 'Expand code' : 'Collapse code');
				collapseBtn.innerHTML = collapsed ? CHEVRON_DOWN_SVG : CHEVRON_UP_SVG;
			});

			actions.append(copyBtn, collapseBtn);
			header.append(langLabel, actions);

			const body = document.createElement('div');
			body.className = 'code-block__body';

			pre.replaceWith(wrapper);
			body.appendChild(pre);
			wrapper.append(header, body);
		});
	}

	$effect(() => {
		// Re-run whenever this message's rendered HTML changes — a brand-new message, or (since
		// the chat page never patches a message in place, only invalidateAll()s the whole list)
		// a full reload replacing every message's DOM at once.
		void message.content_html;
		if (bubbleEl) enhanceCodeBlocks(bubbleEl);
	});

	function copyMessage() {
		navigator.clipboard.writeText(message.content);
		messageCopied = true;
		setTimeout(() => (messageCopied = false), 1500);
	}

	async function shareMessage() {
		if (navigator.share) {
			try {
				await navigator.share({ text: message.content });
			} catch {
				// User dismissed the share sheet — not a failure worth reacting to.
			}
			return;
		}
		// No Web Share API (most desktop browsers) — clipboard is the honest fallback, since this
		// app has no shareable-link/public-session concept to share a URL to instead.
		await navigator.clipboard.writeText(message.content);
		shareCopied = true;
		setTimeout(() => (shareCopied = false), 1500);
	}

	async function setFeedback(rating: 'up' | 'down') {
		const next = feedback === rating ? null : rating; // clicking the active one again retracts it
		const previous = feedback;
		feedback = next;

		const body = new FormData();
		body.set('messageId', message.id);
		if (next) body.set('rating', next);

		const response = await fetch('?/feedback', { method: 'POST', body });
		const result = deserialize(await response.text());
		if (result.type !== 'success') {
			feedback = previous;
		}
	}
</script>

<div class="flex flex-col {message.sender === 'user' ? 'items-end' : 'items-start'} gap-1">
	<div class="flex items-center gap-2 px-1">
		<span class="md-label-medium" style="color: var(--md-sys-color-on-surface)">{senderLabel}</span>
	</div>

	<div
		bind:this={bubbleEl}
		class="message-bubble md-body-large max-w-md px-4 py-2"
		style={message.sender === 'user'
			? `background: var(--md-sys-color-primary-container); color: var(--md-sys-color-on-primary-container); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small) var(--md-sys-shape-corner-large)`
			: `background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)`}
	>
		{@html message.content_html}
	</div>

	<div class="flex items-center gap-1 px-1">
		<span class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{formattedTime}</span>
		<button
			type="button"
			class="message-action-btn"
			aria-label={messageCopied ? 'Copied' : 'Copy message'}
			onclick={copyMessage}
		>
			{#if messageCopied}
				<IconCheck size={16} />
			{:else}
				<IconCopy size={16} />
			{/if}
		</button>
		<button
			type="button"
			class="message-action-btn"
			aria-label={shareCopied ? 'Copied' : 'Share message'}
			onclick={shareMessage}
		>
			{#if shareCopied}
				<IconCheck size={16} />
			{:else}
				<IconShare size={16} />
			{/if}
		</button>
		{#if message.sender === 'assistant'}
			<button
				type="button"
				class="message-action-btn"
				class:message-action-btn--active={feedback === 'up'}
				aria-label="Good response"
				aria-pressed={feedback === 'up'}
				onclick={() => setFeedback('up')}
			>
				<IconThumbUp size={16} />
			</button>
			<button
				type="button"
				class="message-action-btn"
				class:message-action-btn--active={feedback === 'down'}
				aria-label="Bad response"
				aria-pressed={feedback === 'down'}
				onclick={() => setFeedback('down')}
			>
				<IconThumbDown size={16} />
			</button>
		{/if}
	</div>
</div>

<style>
	.message-action-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 28px;
		height: 28px;
		padding: 0;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.message-action-btn:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.message-action-btn--active {
		color: var(--md-sys-color-primary);
	}

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

	/* Shiki's dual-theme output paints its base (light) background as an inline style on <pre>
	   itself, with no per-token background at all normally. The background here is intentionally
	   set ONLY at the <pre> level (never on individual token spans) and forced with !important to
	   beat that inline style — one single background paints the whole block uniformly, so gaps
	   between tokens (whitespace, indentation, line breaks not wrapped in any span) can never show
	   a different color than the syntax-highlighted text around them. */
	.message-bubble :global(pre.shiki) {
		background-color: var(--shiki-light-bg) !important;
	}
	.message-bubble :global(pre.shiki span) {
		background-color: transparent !important;
	}
	@media (prefers-color-scheme: dark) {
		.message-bubble :global(pre.shiki) {
			background-color: var(--shiki-dark-bg) !important;
		}
		.message-bubble :global(.shiki),
		.message-bubble :global(.shiki span) {
			/* Every span carries its own inline color (the light-theme value) — beating that
			   inline style, not just the stylesheet default, needs !important here too, the same
			   as the background-color rule above. Without it, a token whose light-theme color
			   happens to be close to the forced dark background (plain identifiers, typically a
			   muted gray in most themes) renders essentially invisible instead of switching to
			   its --shiki-dark value. */
			color: var(--shiki-dark) !important;
		}
	}

	/* Code block header/collapse chrome — built client-side by enhanceCodeBlocks() above around
	   the server-rendered <pre>, which ends up nested inside .code-block__body. */
	.message-bubble :global(.code-block) {
		margin: 0.5em 0;
		border-radius: var(--md-sys-shape-corner-small);
		overflow: hidden;
		border: 1px solid var(--md-sys-color-outline-variant);
	}
	.message-bubble :global(.code-block pre) {
		margin: 0;
		border-radius: 0;
	}
	.message-bubble :global(.code-block__header) {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 6px 10px;
		background: var(--md-sys-color-surface-container-highest);
		font-family: var(--md-sys-typescale-label-small-font);
		font-size: var(--md-sys-typescale-label-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.message-bubble :global(.code-block__lang) {
		text-transform: uppercase;
		letter-spacing: 0.04em;
	}
	.message-bubble :global(.code-block__actions) {
		display: flex;
		gap: 4px;
	}
	.message-bubble :global(.code-block__btn) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 24px;
		height: 24px;
		padding: 0;
		border: none;
		border-radius: var(--md-sys-shape-corner-extra-small);
		background: transparent;
		color: inherit;
		cursor: pointer;
	}
	.message-bubble :global(.code-block__btn:hover) {
		background: color-mix(in srgb, currentColor 12%, transparent);
	}
	.message-bubble :global(.code-block__body) {
		max-height: 480px;
		overflow: auto;
	}
	.message-bubble :global(.code-block--collapsed .code-block__body) {
		display: none;
	}
</style>
