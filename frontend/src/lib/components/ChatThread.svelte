<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount, type Snippet } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import LoadingIndicator from '$lib/components/m3/LoadingIndicator.svelte';
	import SendButton from '$lib/components/m3/SendButton.svelte';
	import { buildMessageFetchUrl } from '$lib/buildMessageFetchUrl';
	import { agentTypeFallbackLabel, delegationStatusLabel, toolActivityLabel } from '$lib/agentLabels';
	import type { AgentStatus, RenderedMessage } from '$lib/types';

	interface AgentActivityItem {
		id: string;
		target_agent_type: string;
		status: string;
		task: string;
		result: string | null;
		error: string | null;
	}

	let {
		sessionId,
		messages,
		agentActivity,
		agentStatus = null,
		sendError = null,
		extraControls,
	}: {
		sessionId: string;
		messages: RenderedMessage[];
		agentActivity: AgentActivityItem[];
		agentStatus?: AgentStatus | null;
		sendError?: string | null;
		extraControls?: Snippet;
	} = $props();

	let pendingReply = $state(false);
	let submitting = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);
	let activitySheetOpen = $state(false);
	let messagesContainer: HTMLDivElement | undefined = $state();
	let messageInput: HTMLTextAreaElement | undefined = $state();

	let localMessages = $state(messages);

	let currentPhase = $state<{ phase: string; detail: string | null } | null>(
		agentStatus ? { phase: agentStatus.current_phase, detail: agentStatus.current_phase_detail } : null,
	);

	function phaseText(phase: string, detail: string | null): string {
		if (phase === 'thinking') return 'Nomi is thinking…';
		if (phase === 'writing_reply') return 'Nomi is finalizing a reply…';
		if (phase === 'calling_tool') return detail ? `Nomi is ${toolActivityLabel(detail)}…` : 'Nomi is using a tool…';
		return 'Nomi is working…';
	}

	// Resync whenever the page's own `messages` prop changes — navigating to a different
	// session, or a full invalidateAll() (still used for AgentDelegationUpdated and on
	// WS-reconnect-after-drop, see below).
	$effect(() => {
		localMessages = messages;
	});

	// Enter sends, Shift+Enter breaks the line — the textarea grows with its content.
	function onComposerKeydown(event: KeyboardEvent) {
		if (event.key === 'Enter' && !event.shiftKey && !event.isComposing) {
			event.preventDefault();
			const field = event.currentTarget as HTMLTextAreaElement;
			if (field.value.trim() && !isWorking) field.form?.requestSubmit();
		}
	}

	const NIL_UUID = '00000000-0000-0000-0000-000000000000';

	async function fetchAndUpsertMessage(id: string) {
		try {
			// Plain string concatenation via buildMessageFetchUrl, not a bare relative reference —
			// this route never renders with a trailing slash, so `fetch('message/id')` would resolve
			// against the current URL by dropping the sessionId segment (WHATWG relative-URL
			// resolution rules), not appending to it. See buildMessageFetchUrl's own doc comment.
			const response = await fetch(buildMessageFetchUrl(page.url.pathname, id));
			if (!response.ok) return;
			const message: RenderedMessage = await response.json();
			const index = localMessages.findIndex((m) => m.id === message.id);
			if (index === -1) {
				localMessages = [...localMessages, message];
			} else {
				localMessages = [...localMessages.slice(0, index), message, ...localMessages.slice(index + 1)];
			}
		} catch {
			// Best-effort — a dropped connection triggers a full invalidateAll() on reconnect
			// (see the socket 'open' handler below), which reconciles anything missed here.
		}
	}

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	// Consecutive messages from the same sender in the same minute chain into one visual group
	// (no repeated "You"/"Nomi" label, tighter spacing) — same idea as Slack/iMessage grouping.
	function sameMinute(a: string, b: string): boolean {
		const da = new Date(a);
		const db = new Date(b);
		return (
			da.getFullYear() === db.getFullYear() &&
			da.getMonth() === db.getMonth() &&
			da.getDate() === db.getDate() &&
			da.getHours() === db.getHours() &&
			da.getMinutes() === db.getMinutes()
		);
	}

	function isChained(list: RenderedMessage[], index: number): boolean {
		if (index === 0) return false;
		const previous = list[index - 1];
		const current = list[index];
		if (previous.sender !== current.sender) return false;
		// For assistant messages, only chain when the exact same agent produced both — e.g. a
		// money-agent reply immediately followed by a chitchat reply are both "assistant" but
		// must never chain into one unlabeled group.
		if (previous.agent_display_name !== current.agent_display_name) return false;
		return sameMinute(previous.created_at, current.created_at);
	}

	// The sender label shows on the first bubble of a chain (announcing who's talking), but the
	// timestamp shows on the last one instead — same convention as iMessage/Slack: what matters
	// for a timestamp is when the group of messages finished, not when it started.
	function isLastInChain(list: RenderedMessage[], index: number): boolean {
		return index === list.length - 1 || !isChained(list, index + 1);
	}

	const activeDelegationCount = $derived(
		agentActivity.filter((d) => d.status === 'pending' || d.status === 'processing').length,
	);

	// `submitting` covers the gap between hitting Send and the first streamed token arriving —
	// `pendingReply` alone only flips once a WS "Delta" event lands, which leaves a brief window
	// right after sending where neither the button nor the "Typing…" bubble showed any feedback.
	const isWorking = $derived(submitting || pendingReply);

	$effect(() => {
		if (pendingReply || turnError) submitting = false;
	});

	// -1 (not 0) so the very first run — the initial mount — always counts as "grew" and scrolls
	// to the bottom of whatever history loaded, regardless of how many messages that is.
	let lastMessageCount = -1;

	// Keep the latest message (and the "Typing…" indicator) in view, but only when a message was
	// actually appended or a turn is in progress — not on every fetchAndUpsertMessage call. Under
	// Svelte 5, reassigning `localMessages` (even to replace one entry in place, e.g. a todo-list
	// step flipping or an approval being decided after the turn has already finished) re-triggers
	// any effect that reads `.length`, even though the length didn't change. Comparing against the
	// previous count so an in-place `MessageUpdated` patch — which this task's whole point is to
	// apply without visibly jumping the viewport — doesn't yank the scroll position, while a
	// genuinely new message (or live progress while `isWorking`) still does.
	$effect(() => {
		const currentCount = localMessages.length;
		const grew = currentCount > lastMessageCount;
		lastMessageCount = currentCount;
		if (grew || isWorking) {
			messagesContainer?.scrollTo({ top: messagesContainer.scrollHeight });
		}
	});

	onMount(() => {
		messageInput?.focus();

		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = new WebSocket(`/chat/${sessionId}/ws`);

			socket.addEventListener('open', () => {
				retryDelay = INITIAL_RETRY_DELAY_MS;
				connectionLost = false;
				if (hasConnectedBefore) {
					// Reconnected after a drop; neither leg replays missed events, so re-fetch to
					// reconcile anything that happened while disconnected.
					invalidateAll();
				}
				hasConnectedBefore = true;
			});

			socket.addEventListener('message', (event) => {
				let envelope: { kind: string; message_id?: string; phase?: string; detail?: string | null };
				try {
					envelope = JSON.parse(event.data);
				} catch {
					return;
				}
				if (envelope.kind === 'Delta') {
					pendingReply = true;
				} else if (envelope.kind === 'TurnCompleted') {
					pendingReply = false;
					turnError = false;
					if (envelope.message_id && envelope.message_id !== NIL_UUID) {
						fetchAndUpsertMessage(envelope.message_id);
					}
				} else if (envelope.kind === 'TurnFailed') {
					pendingReply = false;
					turnError = true;
				} else if (envelope.kind === 'AgentDelegationUpdated') {
					invalidateAll();
				} else if (envelope.kind === 'MessageCreated' || envelope.kind === 'MessageUpdated') {
					if (envelope.message_id) fetchAndUpsertMessage(envelope.message_id);
				} else if (envelope.kind === 'AgentPhaseChanged') {
					if (typeof envelope.phase === 'string') {
						currentPhase = envelope.phase === 'waiting' ? null : { phase: envelope.phase, detail: envelope.detail ?? null };
					}
				}
			});

			socket.addEventListener('close', (event) => {
				if (intentionallyClosed) return;
				if (TERMINAL_CLOSE_CODES.has(event.code)) {
					connectionLost = true;
					return;
				}
				const jitter = Math.random() * 250;
				retryTimeout = setTimeout(connect, retryDelay + jitter);
				retryDelay = Math.min(retryDelay * 2, MAX_RETRY_DELAY_MS);
			});
		}

		connect();

		return () => {
			intentionallyClosed = true;
			clearTimeout(retryTimeout);
			socket?.close();
		};
	});
</script>

<div class="flex h-full flex-col" style="background: var(--md-sys-color-surface)">
	<div bind:this={messagesContainer} class="thread flex-1 overflow-y-auto">
		<div class="thread__column">
			{#each localMessages as message, i (message.id)}
				<MessageBubble
					{message}
					chained={isChained(localMessages, i)}
					first={i === 0}
					showTimestamp={isLastInChain(localMessages, i)}
				/>
			{/each}
			{#if isWorking}
				<div class="thread__status" role="status">
					<LoadingIndicator size={28} label="Nomi is working" />
					<span>{currentPhase ? phaseText(currentPhase.phase, currentPhase.detail) : 'Nomi is working…'}</span>
				</div>
			{/if}
			{#if turnError}
				<p class="thread__notice" role="alert">Something went wrong — try sending again.</p>
			{/if}
			{#if sendError}
				<p class="thread__notice" role="alert">{sendError}</p>
			{/if}
			{#if connectionLost}
				<p class="thread__notice" role="alert">Couldn't connect to this chat — try reloading the page.</p>
			{/if}
		</div>
	</div>

	<div class="dock">
		{#if activeDelegationCount > 0}
			<button type="button" class="dock__activity" onclick={() => (activitySheetOpen = true)}>
				<AgentShape size={20} working />
				{activeDelegationCount === 1 ? '1 agent working' : `${activeDelegationCount} agents working`}
			</button>
		{/if}
		<form
			method="POST"
			action="?/sendMessage"
			class="composer"
			use:enhance={() => {
				submitting = true;
				return async ({ update }) => {
					await update({ reset: true });
					messageInput?.focus();
				};
			}}
		>
			{#if extraControls}
				<div class="composer__controls">
					{@render extraControls()}
				</div>
			{/if}
			<label for="chat-message" class="sr-only">Message</label>
			<textarea
				id="chat-message"
				bind:this={messageInput}
				name="text"
				rows="1"
				placeholder="Message Nomi"
				required
				class="composer__input"
				onkeydown={onComposerKeydown}
			></textarea>
			<SendButton working={isWorking} />
		</form>
	</div>
</div>

<BottomSheet bind:open={activitySheetOpen}>
	{#snippet children()}
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 12px;">Agent activity</h2>
		{#if agentActivity.length === 0}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No background activity yet.</p>
		{:else}
			{#each agentActivity as item (item.id)}
				<div style="padding: 8px 0; border-bottom: 1px solid var(--md-sys-color-outline-variant);">
					<p class="md-body-large" style="color: var(--md-sys-color-on-surface)">
						{agentTypeFallbackLabel(item.target_agent_type)} — {delegationStatusLabel(item.status)}
					</p>
					<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{item.task}</p>
					{#if item.result}
						<p class="md-body-small" style="color: var(--md-sys-color-on-surface-variant)">{item.result}</p>
					{/if}
					{#if item.error}
						<p class="md-body-small" style="color: var(--md-sys-color-error)">{item.error}</p>
					{/if}
				</div>
			{/each}
		{/if}
	{/snippet}
</BottomSheet>

<style>
	.thread {
		padding: 24px clamp(16px, 3vw, 40px) 8px;
	}
	.thread__column {
		max-width: 800px;
		margin: 0 auto;
	}
	.thread__status {
		display: flex;
		align-items: center;
		gap: 12px;
		margin-top: 20px;
		padding-left: 4px;
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-medium-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	.thread__notice {
		margin: 16px 0 0;
		padding: 12px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}

	.dock {
		padding: 8px clamp(12px, 3vw, 40px) 16px;
	}
	.dock__activity {
		display: flex;
		align-items: center;
		gap: 8px;
		max-width: 800px;
		margin: 0 auto 8px;
		height: 36px;
		padding: 0 14px 0 10px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-family: var(--md-sys-typescale-label-large-font);
		font-size: var(--md-sys-typescale-label-large-size);
		font-weight: 600;
		cursor: pointer;
	}
	.composer {
		display: flex;
		align-items: flex-end;
		gap: 6px;
		max-width: 800px;
		margin: 0 auto;
		padding: 8px 8px 8px 10px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		box-shadow:
			0 1px 0 var(--md-sys-color-outline-variant),
			0 18px 40px -28px color-mix(in srgb, var(--md-sys-color-on-surface) 45%, transparent);
	}
	.composer__controls {
		display: flex;
		align-items: center;
		align-self: center;
		flex: none;
	}
	.composer__input {
		flex: 1;
		min-width: 0;
		align-self: center;
		max-height: 200px;
		field-sizing: content;
		padding: 12px 6px;
		border: none;
		resize: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: var(--md-sys-typescale-body-large-size);
		line-height: 1.5;
	}
	.composer__input::placeholder {
		color: var(--md-sys-color-on-surface-variant);
	}
</style>
