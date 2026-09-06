<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { onMount, type Snippet } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import type { RenderedMessage } from '$lib/types';

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
		sendError = null,
		extraControls,
	}: {
		sessionId: string;
		messages: RenderedMessage[];
		agentActivity: AgentActivityItem[];
		sendError?: string | null;
		extraControls?: Snippet;
	} = $props();

	let pendingReply = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);
	let activitySheetOpen = $state(false);
	let messagesContainer: HTMLDivElement | undefined = $state();
	let messageInput: HTMLInputElement | undefined = $state();

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	// Consecutive messages from the same sender, close enough in time, chain into one visual
	// group (no repeated "You"/"Nomi" label, tighter spacing) — same idea as Slack/iMessage
	// grouping. 5 minutes is a common grouping window for that pattern.
	const CHAIN_WINDOW_MS = 5 * 60 * 1000;

	function isChained(list: RenderedMessage[], index: number): boolean {
		if (index === 0) return false;
		const previous = list[index - 1];
		const current = list[index];
		if (previous.sender !== current.sender) return false;
		const gapMs = new Date(current.created_at).getTime() - new Date(previous.created_at).getTime();
		return gapMs <= CHAIN_WINDOW_MS;
	}

	const activeDelegationCount = $derived(
		agentActivity.filter((d) => d.status === 'pending' || d.status === 'processing').length,
	);

	// Always keep the latest message (and the "Typing…" indicator) in view — re-runs whenever
	// the message list changes or a reply starts streaming.
	$effect(() => {
		messages.length;
		pendingReply;
		messagesContainer?.scrollTo({ top: messagesContainer.scrollHeight });
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
				let envelope: { kind: string };
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
					invalidateAll();
				} else if (envelope.kind === 'TurnFailed') {
					pendingReply = false;
					turnError = true;
				} else if (envelope.kind === 'AgentDelegationUpdated') {
					invalidateAll();
				} else if (envelope.kind === 'SessionActivity') {
					invalidateAll();
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
	<div bind:this={messagesContainer} class="flex-1 overflow-y-auto px-6 py-6">
		{#each messages as message, i (message.id)}
			<MessageBubble {message} chained={isChained(messages, i)} first={i === 0} />
		{/each}
		{#if pendingReply}
			<div class="mt-4 flex justify-start">
				<div
					class="md-body-large max-w-md px-4 py-2"
					style="background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface-variant); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)"
				>
					Typing…
				</div>
			</div>
		{/if}
		{#if turnError}
			<p class="md-body-medium mt-4 text-center" style="color: var(--md-sys-color-error)">
				Something went wrong — try sending again.
			</p>
		{/if}
		{#if sendError}
			<p class="md-body-medium mt-4 text-center" style="color: var(--md-sys-color-error)">{sendError}</p>
		{/if}
		{#if connectionLost}
			<p class="md-body-medium mt-4 text-center" style="color: var(--md-sys-color-error)">
				Couldn't connect to this chat — try reloading the page.
			</p>
		{/if}
	</div>

	<div
		class="px-6 py-4"
		style="background: var(--md-sys-color-surface-container-low); border-top: 1px solid var(--md-sys-color-outline-variant)"
	>
		{#if activeDelegationCount > 0}
			<div class="mb-2 flex items-center">
				<button
					type="button"
					class="md-label-medium"
					style="color: var(--md-sys-color-primary); background: none; border: none; cursor: pointer; padding: 4px 8px;"
					onclick={() => (activitySheetOpen = true)}
				>
					{activeDelegationCount === 1 ? '1 agent working…' : `${activeDelegationCount} agents working…`}
				</button>
			</div>
		{/if}
		<form
			method="POST"
			action="?/sendMessage"
			use:enhance={() => {
				return async ({ update }) => {
					await update({ reset: true });
					messageInput?.focus();
				};
			}}
		>
			<div
				class="flex items-center gap-2 px-4 py-2"
				style="background: var(--md-sys-color-surface); border-radius: var(--md-sys-shape-corner-full); border: 1px solid var(--md-sys-color-outline)"
			>
				<input
					bind:this={messageInput}
					name="text"
					type="text"
					placeholder="Ask me anything..."
					required
					class="md-body-large flex-1 border-none bg-transparent outline-none"
					style="color: var(--md-sys-color-on-surface)"
				/>
				<Button type="submit" variant="filled">Send</Button>
			</div>
		</form>
		{#if extraControls}
			<div class="mt-2 flex items-center gap-1">
				{@render extraControls()}
			</div>
		{/if}
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
						{item.target_agent_type} — {item.status}
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
