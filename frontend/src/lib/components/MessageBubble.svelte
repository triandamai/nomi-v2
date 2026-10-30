<script lang="ts">
	import { untrack } from 'svelte';
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
	import IconDelete from './icons/IconDelete.svelte';
	import MessageDeleteSheet from './MessageDeleteSheet.svelte';
	import MessageMemoriesSheet from './MessageMemoriesSheet.svelte';
	import type { RenderedMessage } from '$lib/types';
	import ResponseActions from './ds/chat/ResponseActions.svelte';
	import DislikeFeedback, { type DislikeReason } from './ds/chat/DislikeFeedback.svelte';
	import FeedbackSent from './ds/chat/FeedbackSent.svelte';
	import ShareTwin from './blocks/twins/ShareTwin.svelte';
	import RegenerateMenu from './ds/chat/RegenerateMenu.svelte';
	import MessageEditor from './ds/chat/MessageEditor.svelte';
	import VersionSwitcher from './ds/chat/VersionSwitcher.svelte';
	import { Pencil } from '@lucide/svelte';
	import { page } from '$app/state';
	import type { MessageVersion } from '$lib/types';

	let {
		message,
		chained = false,
		first = false,
		showTimestamp = true,
		reasoning = [],
		thinkingOnly = false,
		thread = 'solo',
		handoffFrom = null,
		answered = false,
		ondelete,
		onrerun,
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
		/** The person has replied since this message (a questions card shows as answered). */
		answered?: boolean;
		/** Offers Delete; called with the ids of the messages that went. */
		ondelete?: (ids: string[]) => void;
		/** The latest turn ran again (edited, or a reply asked for again): the person's message as it now stands. */
		onrerun?: (startId: string, update: Partial<RenderedMessage>) => void;
	} = $props();
	const steps = $derived(message.steps ?? []);
	// Buttons or an error bubble the crew posted on their own: no copy, share or rating for them,
	// and next-step buttons go once the person has moved on.
	const blockKinds = $derived((message.content_blocks ?? []).map((block) => block.kind));
	const onlyBlocks = (kinds: string[]) => blockKinds.length > 0 && blockKinds.every((kind) => kinds.includes(kind));
	const statusOnly = $derived(onlyBlocks(['suggestions', 'run_failed']));
	const spentSuggestions = $derived(answered && onlyBlocks(['suggestions']));
	// Arrived in the last half minute (live, not from history): file cards open so the change is seen.
	const fresh = untrack(() => Date.now() - Date.parse(message.created_at) < 30_000);
	const sources = $derived(message.sources ?? []);
	const continues = $derived(thread === 'start' || thread === 'middle');
	const inThread = $derived(thread !== 'solo');

	let bubbleEl: HTMLDivElement | undefined = $state();
	// The files a user attached are referenced in the message text (see $lib/attachments).
	const attached = $derived(message.sender === 'user' ? splitAttachments(message.content) : null);
	let feedback = $state(untrack(() => message.my_feedback));
	let messageCopied = $state(false);
	let shareCopied = $state(false);
	let deleteOpen = $state(false);

	const senderLabel = $derived(message.sender === 'user' ? m.msg_you() : (message.agent_display_name ?? 'Nomi'));
	// Crew members other than Nomi get their name as a chip tinted with their gradient, so a
	// hand-off reads at a glance (matches the agent's avatar shape).
	const look = $derived(agentLook(message.agent_display_name));
	const isCrewMember = $derived(message.sender === 'assistant' && look.tone !== 'glow');
	const senderTint = $derived(GRADIENT_STOPS[look.tone].at(-1));
	const formattedTime = $derived(
		new Date(message.created_at).toLocaleString(getLocale(), {
			dateStyle: 'medium',
			timeStyle: 'short',
		}),
	);

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
		// No Web Share API (most desktop browsers): the clipboard. (A reply shares by link instead,
		// through ShareTwin.)
		await navigator.clipboard.writeText(message.content);
		shareCopied = true;
		setTimeout(() => (shareCopied = false), 1500);
	}

	// The memories this reply drew on ("Used 2 memories").
	let memoriesOpen = $state(false);
	// After a thumbs-down: what went wrong (DislikeFeedback), then what was sent (FeedbackSent).
	let dislikeOpen = $state(false);
	let dislikeSent = $state<{ withChat: boolean } | null>(null);
	// Sharing this reply, or its chat, by link (ShareTwin).
	let shareOpen = $state(false);

	// The latest turn can run again: the person edits their message, or asks for the reply again
	// (see message_rerun on the server). Its earlier takes stay, a step back with the version switcher.
	const latest = $derived(!answered && !message.author_name && !!onrerun);
	const messageUrl = $derived(`${page.url.pathname}/message/${message.id}`);
	let regenerateOpen = $state(false);
	let editing = $state(false);
	let rerunError = $state(false);

	async function rerun(request: Promise<Response>): Promise<{ message_id: string; total: number; edited_at: string | null } | null> {
		const response = await request.catch(() => null);
		rerunError = !response?.ok;
		if (!response?.ok) return null;
		// Back to the live take; the list of earlier ones has grown.
		viewing = null;
		earlier = null;
		return await response.json();
	}

	async function regenerate(choice: { style: 'again' | 'shorter' | 'longer' | 'list' } | { model: string }) {
		regenerateOpen = false;
		const style = 'style' in choice ? choice.style : 'again';
		const done = await rerun(fetch(`${messageUrl}/regenerate`, { method: 'POST', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ style }) }));
		if (done) onrerun?.(done.message_id, { versions: done.total - 1 });
	}

	async function saveEdit(text: string) {
		const done = await rerun(fetch(messageUrl, { method: 'PATCH', headers: { 'content-type': 'application/json' }, body: JSON.stringify({ text }) }));
		if (!done) return;
		editing = false;
		const rendered = await fetch(messageUrl).then((response) => (response.ok ? (response.json() as Promise<RenderedMessage>) : null)).catch(() => null);
		onrerun?.(done.message_id, rendered ?? { content: text, edited_at: done.edited_at, versions: done.total - 1 });
	}

	// Stepping through the earlier takes of this message: `viewing` is the take shown (null: the live one).
	const total = $derived((message.versions ?? 0) + 1);
	let viewing = $state<number | null>(null);
	let earlier = $state<MessageVersion[] | null>(null);
	const shown = $derived(viewing !== null && earlier ? earlier[viewing - 1] : null);

	async function step(by: number) {
		const next = (viewing ?? total) + by;
		if (next < 1 || next > total) return;
		if (next < total && !earlier) {
			const response = await fetch(`${messageUrl}/versions`).catch(() => null);
			if (!response?.ok) {
				rerunError = true;
				return;
			}
			earlier = ((await response.json()) as { versions: MessageVersion[] }).versions;
		}
		viewing = next === total ? null : next;
	}

	type Dislike = { reasons: DislikeReason[]; note: string; includeChat: boolean };

	async function sendFeedback(rating: 'up' | 'down' | null, dislike?: Dislike): Promise<boolean> {
		const body = new FormData();
		body.set('messageId', message.id);
		if (rating) body.set('rating', rating);
		if (dislike) {
			for (const reason of dislike.reasons) body.append('reasons', reason);
			body.set('note', dislike.note);
			body.set('includeChat', String(dislike.includeChat));
		}
		const response = await fetch('?/feedback', { method: 'POST', body });
		return deserialize(await response.text()).type === 'success';
	}

	async function sendDislike(dislike: Dislike) {
		if (await sendFeedback('down', dislike)) {
			dislikeOpen = false;
			dislikeSent = { withChat: dislike.includeChat };
		}
	}

	async function setFeedback(rating: 'up' | 'down') {
		const next = feedback === rating ? null : rating; // clicking the active one again retracts it
		const previous = feedback;
		feedback = next;
		if (!(await sendFeedback(next))) {
			feedback = previous;
			return;
		}
		dislikeOpen = next === 'down';
		dislikeSent = null;
	}

	function showMemories() {
		memoriesOpen = true;
	}
</script>

{#if !spentSuggestions}
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

		{#if message.sender === 'user' && message.author_name}
			<span class="message__who message__who--teammate">{message.author_name}</span>
		{/if}

		{#if reasoning.length > 0 || steps.length > 0 || sources.length > 0}
			<ActivityDisclosure thinking={reasoning} {steps} {sources} live={thinkingOnly} />
		{/if}

		{#if !thinkingOnly && editing}
			<MessageEditor value={message.content} version={total > 1 ? `${total} / ${total}` : undefined} onsave={saveEdit} oncancel={() => (editing = false)} />
		{:else if !thinkingOnly}
		<div bind:this={bubbleEl} class="message-bubble md-body-large" class:message-bubble--user={message.sender === 'user'} class:message-bubble--teammate={!!message.author_name}>
			{#if message.content_blocks && message.content_blocks.length > 0}
				<div class="flex flex-col gap-3">
					{#each message.content_blocks as block, i (i)}
						<ContentBlockView {block} messageId={message.id} agent={message.agent_display_name} {fresh} {answered} text={message.content} onremove={() => ondelete?.([message.id])} />
					{/each}
				</div>
			{:else if attached && attached.files.length > 0}
				<MessageAttachments files={attached.files} />
				{#if attached.text}<p class="message__plain">{attached.text}</p>{/if}
			{:else if shown}
				<p class="message__plain">{shown.text}</p>
			{:else}
				<div class="message-markdown">{@html message.content_html}</div>
			{/if}
		</div>
		{#if total > 1}
			<div class="message__versions">
				<VersionSwitcher current={viewing ?? total} {total} onprev={() => step(-1)} onnext={() => step(1)} />
				{#if message.edited_at && viewing === null}<span class="message__edited">{m.msg_edited({ time: new Date(message.edited_at).toLocaleTimeString(getLocale(), { hour: '2-digit', minute: '2-digit' }) })}</span>{/if}
			</div>
		{/if}
		{#if shown}
			<div class="message__earlier" role="group" aria-label={m.msg_earlier_version({ version: viewing ?? total, total })}>
				<p class="message__earlier-label">{m.msg_earlier_version({ version: viewing ?? total, total })}</p>
				{#each shown.replies as reply, index (index)}
					<div class="message__earlier-reply">
						{#if reply.agent_display_name}<span class="message__earlier-name">{reply.agent_display_name}</span>{/if}
						<div class="message-markdown">{@html reply.content_html}</div>
					</div>
				{/each}
			</div>
		{/if}

		{#if !statusOnly && message.sender === 'assistant'}
		<div class="message__reply-actions">
			<ResponseActions
				text={message.content}
				meta={showTimestamp ? formattedTime : undefined}
				feedback={feedback === 'up' ? 'liked' : feedback === 'down' ? 'disliked' : null}
				onlike={() => setFeedback('up')}
				ondislike={() => setFeedback('down')}
				onshare={() => (shareOpen = true)}
				onregenerate={latest ? () => (regenerateOpen = !regenerateOpen) : undefined}
				ondelete={ondelete && !message.author_name ? () => (deleteOpen = true) : undefined}
			/>
		</div>
		{#if regenerateOpen}<RegenerateMenu inline onpick={regenerate} />{/if}
		{#if rerunError}<p class="message__error" role="alert">{m.msg_rerun_failed()}</p>{/if}
		<ShareTwin bind:open={shareOpen} messageId={message.id} agent={message.agent_display_name} time={formattedTime} />
		{#if dislikeOpen}
			<DislikeFeedback onsend={sendDislike} oncancel={() => (dislikeOpen = false)} />
		{:else if dislikeSent}
			<FeedbackSent withChat={dislikeSent.withChat} />
		{/if}
		{:else if !statusOnly}
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
			{#if latest && !(attached && attached.files.length > 0)}
				<button type="button" class="message-action-btn" aria-label={m.msg_edit()} title={m.msg_edit()} onclick={() => (editing = true)}>
					<Pencil size={16} strokeWidth={1.75} aria-hidden="true" />
				</button>
			{/if}
			{#if ondelete && !message.author_name}
				<button type="button" class="message-action-btn" aria-label={m.msg_delete()} title={m.msg_delete()} onclick={() => (deleteOpen = true)}>
					<IconDelete size={16} />
				</button>
			{/if}
			{#if showTimestamp}
				<span class="message__time">{formattedTime}</span>
			{/if}
		</div>
		{/if}
		{#if message.sender === 'assistant' && message.memory_count > 0}
			<button type="button" class="message-memories" onclick={showMemories} aria-label={m.used_chip_label({ count: message.memory_count })}>
				<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><circle cx="6" cy="6" r="3" /><circle cx="18" cy="8" r="3" /><circle cx="10" cy="18" r="3" /><path d="M8.5 7.5 15 8M7.5 8.5l1.7 7M16.4 10.4l-4.6 5.4" /></svg>
				{message.memory_count === 1 ? m.used_chip_one() : m.used_chip_many({ count: message.memory_count })}
			</button>
		{/if}
		{/if}
	</div>
</div>
{/if}

{#if ondelete}
	<MessageDeleteSheet bind:open={deleteOpen} messageId={message.id} ondeleted={ondelete} />
{/if}

{#if message.sender === 'assistant'}
	<MessageMemoriesSheet
		bind:open={memoriesOpen}
		messageId={message.id}
		memoryCount={message.memory_count ?? 0}
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
		gap: 12px;
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
	/* A teammate's message in a shared project chat: their name, and a neutral bubble. */
	.message__who--teammate {
		align-self: flex-end;
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.message-bubble--user.message-bubble--teammate {
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface);
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
		font-size: 1rem;
		line-height: 1.6;
		overflow-wrap: anywhere;
	}
	.message-bubble--user {
		align-self: auto;
		padding: 14px 16px;
		font-size: 0.9375rem;
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
	.message__reply-actions {
		margin-top: 2px;
	}
	.message__versions {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.message--user .message__versions {
		align-self: flex-end;
	}
	.message__edited {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.message__earlier {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 12px 14px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
	}
	.message__earlier-label,
	.message__earlier-name {
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.6875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.message__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: var(--md-sys-typescale-body-small-size);
	}

	/* File cards and image previews (MessageAttachments) aren't text links. */
	.message-bubble :global(a.attachment-link) {
		text-decoration: none;
	}

</style>
