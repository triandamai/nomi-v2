<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { deserialize } from '$app/forms';
	import AgentShape from './m3/AgentShape.svelte';
	import { agentLook, GRADIENT_STOPS } from './m3/shapes';
	import ContentBlockView from './blocks/ContentBlockView.svelte';
	import ActivityDisclosure from './blocks/ActivityDisclosure.svelte';
	import { splitAttachments } from '$lib/attachments';
	import MessageAttachments from './MessageAttachments.svelte';
	import IconCheck from './icons/IconCheck.svelte';
	import IconCopy from './icons/IconCopy.svelte';
	import IconShare from './icons/IconShare.svelte';
	import IconThumbDown from './icons/IconThumbDown.svelte';
	import IconThumbUp from './icons/IconThumbUp.svelte';
	import MessageMemoriesSheet from './MessageMemoriesSheet.svelte';
	import type { FeedbackReason, RenderedMessage } from '$lib/types';

	let {
		message,
		chained = false,
		first = false,
		showTimestamp = true,
		reasoning = [],
		thinkingOnly = false,
		thread = 'solo',
		handoffFrom = null,
	}: {
		message: RenderedMessage;
		chained?: boolean;
		first?: boolean;
		showTimestamp?: boolean;
		/** Thinking that led to this message, shown collapsed above it (see $lib/reasoning). */
		reasoning?: string[];
		/** The message is only thinking so far: show the collapsed thinking and nothing else. */
		thinkingOnly?: boolean;
		/**
		 * Where this reply sits among the crew's replies to one message: they read as one thread,
		 * joined by a line through their avatars ('start' and 'middle' continue to the next).
		 */
		thread?: 'solo' | 'start' | 'middle' | 'end';
		/** The crew member who passed the conversation to this one, in the same thread. */
		handoffFrom?: string | null;
	} = $props();
	const steps = $derived(message.steps ?? []);
	const sources = $derived(message.sources ?? []);
	const continues = $derived(thread === 'start' || thread === 'middle');
	const inThread = $derived(thread !== 'solo');

	let bubbleEl: HTMLDivElement | undefined = $state();
	// The files a user attached are referenced in the message text (see $lib/attachments).
	const attached = $derived(message.sender === 'user' ? splitAttachments(message.content) : null);
	let feedback = $state(message.my_feedback);
	let messageCopied = $state(false);
	let shareCopied = $state(false);

	const senderLabel = message.sender === 'user' ? m.msg_you() : (message.agent_display_name ?? 'Nomi');
	// Crew members other than Nomi get their name as a chip tinted with their gradient, so a
	// hand-off reads at a glance (matches the agent's avatar shape).
	const look = $derived(agentLook(message.agent_display_name));
	const isCrewMember = $derived(message.sender === 'assistant' && look.tone !== 'glow');
	const senderTint = $derived(GRADIENT_STOPS[look.tone].at(-1));
	const formattedTime = new Date(message.created_at).toLocaleString(getLocale(), {
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
			copyBtn.setAttribute('aria-label', m.msg_copy_code());
			copyBtn.innerHTML = COPY_SVG;
			copyBtn.addEventListener('click', () => {
				navigator.clipboard.writeText(pre.textContent ?? '');
				copyBtn.innerHTML = CHECK_SVG;
				copyBtn.setAttribute('aria-label', m.msg_copied());
				setTimeout(() => {
					copyBtn.innerHTML = COPY_SVG;
					copyBtn.setAttribute('aria-label', m.msg_copy_code());
				}, 1500);
			});

			const collapseBtn = document.createElement('button');
			collapseBtn.type = 'button';
			collapseBtn.className = 'code-block__btn';
			collapseBtn.setAttribute('aria-label', m.msg_collapse_code());
			collapseBtn.setAttribute('aria-expanded', 'true');
			collapseBtn.innerHTML = CHEVRON_UP_SVG;
			collapseBtn.addEventListener('click', () => {
				const collapsed = wrapper.classList.toggle('code-block--collapsed');
				collapseBtn.setAttribute('aria-expanded', String(!collapsed));
				collapseBtn.setAttribute('aria-label', collapsed ? m.msg_expand_code() : m.msg_collapse_code());
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

	// A reference-style citation ("[1] Attention is all you need — [1]: https://arxiv.org/...")
	// renders through `marked` as a plain `<a href="...">1</a>` — no different from any other
	// link. The heuristic: a citation's visible text is exactly its numeric label; a normal
	// inline link's text is a real phrase. Not airtight (a link whose text happens to be a bare
	// number also matches), but cheap and correct for the actual citation format this is for.
	function enhanceCitations(root: HTMLElement) {
		const links = root.querySelectorAll<HTMLAnchorElement>('a[href]');
		links.forEach((link) => {
			const text = link.textContent?.trim() ?? '';
			if (/^\d+$/.test(text)) {
				link.classList.add('citation-chip');
			}
		});
	}

	$effect(() => {
		// Re-run whenever this message's rendered HTML changes — a brand-new message, or (since
		// the chat page never patches a message in place, only invalidateAll()s the whole list)
		// a full reload replacing every message's DOM at once.
		void message.content_html;
		if (bubbleEl) {
			enhanceCodeBlocks(bubbleEl);
			enhanceCitations(bubbleEl);
		}
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

	// The memories this reply drew on, and "What was off?" after a thumbs-down.
	let memoriesOpen = $state(false);
	let askReason = $state(false);

	async function sendFeedback(rating: 'up' | 'down' | null, reason?: FeedbackReason): Promise<boolean> {
		const body = new FormData();
		body.set('messageId', message.id);
		if (rating) body.set('rating', rating);
		if (reason) body.set('reason', reason);
		const response = await fetch('?/feedback', { method: 'POST', body });
		return deserialize(await response.text()).type === 'success';
	}

	async function setFeedback(rating: 'up' | 'down') {
		const next = feedback === rating ? null : rating; // clicking the active one again retracts it
		const previous = feedback;
		feedback = next;
		if (!(await sendFeedback(next))) {
			feedback = previous;
			return;
		}
		if (next === 'down') {
			askReason = true;
			memoriesOpen = true;
		}
	}

	function showMemories() {
		askReason = false;
		memoriesOpen = true;
	}
</script>

<div
	class="message"
	class:message--user={message.sender === 'user'}
	class:message--threaded={inThread}
	class:message--continues={continues}
	class:message--chained={chained}
	class:message--start={thread === 'start'}
	style="margin-top: {first ? '0' : chained ? '2px' : inThread && thread !== 'start' ? '6px' : '14px'}"
>
	{#if message.sender !== 'user'}
		<div class="message__avatar">
			{#if !chained}
				<AgentShape agent={message.agent_display_name} size={36} face />
			{/if}
		</div>
	{/if}

	<div class="message__body">
		{#if !chained && message.sender !== 'user'}
			<span class="message__who">
				{#if isCrewMember}
					<span class="message__sender message__sender--chip" style="--tint: {senderTint}">{senderLabel}</span>
				{:else}
					<span class="message__sender">{senderLabel}</span>
				{/if}
				{#if handoffFrom}
					<span class="message__handoff">
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M4 4v7a4 4 0 0 0 4 4h12" /><path d="m15 10 5 5-5 5" /></svg>
						{m.thread_handoff({ name: handoffFrom })}
					</span>
				{/if}
			</span>
		{/if}

		{#if reasoning.length > 0 || steps.length > 0 || sources.length > 0}
			<ActivityDisclosure thinking={reasoning} {steps} {sources} live={thinkingOnly} />
		{/if}

		{#if !thinkingOnly}
		<div bind:this={bubbleEl} class="message-bubble md-body-large" class:message-bubble--user={message.sender === 'user'}>
			{#if message.content_blocks && message.content_blocks.length > 0}
				<div class="flex flex-col gap-3">
					{#each message.content_blocks as block, i (i)}
						<ContentBlockView {block} messageId={message.id} agent={message.agent_display_name} />
					{/each}
				</div>
			{:else if attached && attached.files.length > 0}
				<MessageAttachments files={attached.files} />
				{#if attached.text}<p class="message__plain">{attached.text}</p>{/if}
			{:else}
				{@html message.content_html}
			{/if}
		</div>

		<div class="message__actions">
			<button
				type="button"
				class="message-action-btn"
				aria-label={messageCopied ? m.msg_copied() : m.msg_copy()}
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
				aria-label={shareCopied ? m.msg_copied() : m.msg_share()}
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
					aria-label={m.msg_good()}
					aria-pressed={feedback === 'up'}
					onclick={() => setFeedback('up')}
				>
					<IconThumbUp size={16} />
				</button>
				<button
					type="button"
					class="message-action-btn"
					class:message-action-btn--active={feedback === 'down'}
					aria-label={m.msg_bad()}
					aria-pressed={feedback === 'down'}
					onclick={() => setFeedback('down')}
				>
					<IconThumbDown size={16} />
				</button>
			{/if}
			{#if showTimestamp}
				<span class="message__time">{formattedTime}</span>
			{/if}
		</div>
		{#if message.sender === 'assistant' && message.memory_count > 0}
			<button type="button" class="message-memories" onclick={showMemories} aria-label={m.used_chip_label({ count: message.memory_count })}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="6" cy="6" r="3" /><circle cx="18" cy="8" r="3" /><circle cx="10" cy="18" r="3" /><path d="M8.5 7.5 15 8M7.5 8.5l1.7 7M16.4 10.4l-4.6 5.4" /></svg>
				{message.memory_count === 1 ? m.used_chip_one() : m.used_chip_many({ count: message.memory_count })}
			</button>
		{/if}
		{/if}
	</div>
</div>

{#if message.sender === 'assistant'}
	<MessageMemoriesSheet
		bind:open={memoriesOpen}
		messageId={message.id}
		memoryCount={message.memory_count ?? 0}
		{askReason}
		onreason={(reason) => sendFeedback('down', reason)}
	/>
{/if}

<style>
	/* Always visible (unlike the hover actions): which memories shaped a reply is worth seeing. */
	.message-memories {
		align-self: flex-start;
		margin-top: 2px;
		display: inline-flex;
		align-items: center;
		gap: 6px;
		min-height: 32px;
		padding: 0 10px;
		border: none;
		border-radius: 999px;
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		font: inherit;
		font-size: 0.75rem;
		font-weight: 600;
		cursor: pointer;
	}
	.message-memories:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
		color: var(--md-sys-color-on-surface);
	}
	.message-memories:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	/* Assistant turns are editorial — no bubble, the agent's shape as the avatar — so long
	   answers read like a page. The user's own turns are compact bubbles whose tight corner
	   points back at them. */
	.message {
		display: flex;
		gap: 14px;
		align-items: flex-start;
	}
	.message--user {
		justify-content: flex-end;
	}
	.message__avatar {
		position: relative;
		flex: none;
		align-self: stretch;
		width: 36px;
	}
	/* The crew's replies to one message read as one thread: a line runs through the avatars from
	   each reply to the next, so a hand-off looks like the crew passing the work along. */
	.message--threaded .message__avatar::before,
	.message--continues .message__avatar::after {
		content: '';
		position: absolute;
		left: 17px;
		width: 2px;
		border-radius: 1px;
		background: color-mix(in srgb, var(--md-sys-color-primary) 28%, var(--md-sys-color-outline-variant));
	}
	/* Joining from the reply above (not on the first one). */
	.message--threaded:not(.message--start) .message__avatar::before {
		top: -8px;
		height: 8px;
	}
	.message--start .message__avatar::before {
		display: none;
	}
	/* On to the next reply: from under the avatar, or the whole height where there's none. */
	.message--continues .message__avatar::after {
		top: 40px;
		bottom: -8px;
	}
	.message--continues.message--chained .message__avatar::after,
	.message--chained.message--threaded:not(.message--start) .message__avatar::before {
		top: -8px;
	}
	.message--chained.message--threaded:not(.message--continues) .message__avatar::before {
		height: 26px;
	}
	.message__who {
		display: inline-flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 8px;
	}
	.message__handoff {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		animation: handoff-in var(--nomi-motion-spatial-default, 350ms) both;
	}
	@keyframes handoff-in {
		from {
			opacity: 0;
			transform: translateX(-6px);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		.message__handoff {
			animation: none;
		}
	}
	.message__plain {
		margin: 0;
		white-space: pre-wrap;
	}
	.message__body {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 8px;
		min-width: 0;
		flex: 1;
	}
	.message--user .message__body {
		flex: 0 1 auto;
		align-items: flex-end;
		max-width: 78%;
	}
	.message__sender {
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		font-weight: 700;
		color: var(--md-sys-color-on-surface);
		line-height: 36px;
	}
	.message__sender--chip {
		display: inline-flex;
		align-items: center;
		height: 28px;
		margin-top: 4px;
		padding: 0 12px;
		border-radius: var(--md-sys-shape-corner-full);
		background: color-mix(in srgb, var(--tint) 24%, var(--md-sys-color-surface-container-lowest));
		line-height: 1;
		font-size: 0.8125rem;
	}
	.message-bubble {
		align-self: stretch;
		color: var(--md-sys-color-on-surface);
		font-size: 1.0625rem;
		line-height: 1.6;
		overflow-wrap: anywhere;
	}
	.message-bubble--user {
		align-self: auto;
		padding: 16px 20px;
		font-size: 1rem;
		border-radius: var(--nomi-shape-bubble-end);
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
		line-height: 1.5;
	}
	.message__actions {
		display: flex;
		align-items: center;
		gap: 2px;
		margin-top: -4px;
		opacity: 0;
		transition: opacity var(--nomi-motion-effects-fast);
	}
	/* Touch screens have no hover — keep the actions quietly visible there. */
	@media (hover: none) {
		.message__actions {
			opacity: 0.6;
		}
	}
	.message:hover .message__actions,
	.message:focus-within .message__actions {
		opacity: 1;
	}
	.message__time {
		margin-left: 6px;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		letter-spacing: 0.04em;
		color: var(--md-sys-color-on-surface-variant);
	}

	.message-action-btn {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		padding: 0;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
		transition: border-radius var(--nomi-motion-spatial-fast), background-color var(--nomi-motion-effects-fast);
	}
	.message-action-btn:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.message-action-btn--active {
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
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
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.85em;
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
		margin: 0.75em 0;
		padding: 10px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container);
	}
	.message-bubble :global(a) {
		color: inherit;
		text-decoration: underline;
	}
	/* File cards and image previews (MessageAttachments) aren't text links. */
	.message-bubble :global(a.attachment-link) {
		text-decoration: none;
	}
	.message-bubble :global(a.citation-chip) {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		min-width: 1.2em;
		height: 1.2em;
		padding: 0 0.3em;
		margin: 0 0.1em;
		border-radius: var(--md-sys-shape-corner-full);
		background: color-mix(in srgb, var(--md-sys-color-primary) 15%, transparent);
		color: var(--md-sys-color-primary);
		font-size: 0.7em;
		font-weight: 600;
		text-decoration: none;
		vertical-align: super;
	}
	.message-bubble :global(table) {
		border-collapse: collapse;
		margin: 0.75em 0;
		border-radius: var(--md-sys-shape-corner-large-increased);
		overflow: hidden;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.message-bubble :global(th) {
		background: var(--md-sys-color-surface-container);
		text-align: left;
		font-weight: 650;
	}
	.message-bubble :global(th),
	.message-bubble :global(td) {
		border-top: 1px solid var(--md-sys-color-surface-container);
		padding: 10px 16px;
		overflow-wrap: normal;
	}
	/* Wide tables scroll inside their own rounded box instead of squeezing words apart. */
	.message-bubble :global(table) {
		display: block;
		max-width: 100%;
		overflow-x: auto;
		width: fit-content;
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
		margin: 0.75em 0;
		border-radius: var(--md-sys-shape-corner-large-increased);
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
