<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let pendingReply = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	onMount(() => {
		let socket: WebSocket | undefined;
		let retryDelay = INITIAL_RETRY_DELAY_MS;
		let retryTimeout: ReturnType<typeof setTimeout> | undefined;
		let intentionallyClosed = false;
		let hasConnectedBefore = false;

		function connect() {
			socket = new WebSocket(`/chat/${page.params.sessionId}/ws`);

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

<div class="flex h-full flex-col">
	<div class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each data.messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if pendingReply}
			<div class="flex justify-start">
				<div class="max-w-md rounded-2xl bg-neutral-100 px-4 py-2 text-neutral-400">Typing…</div>
			</div>
		{/if}
		{#if turnError}
			<p class="text-center text-sm text-red-600">Something went wrong — try sending again.</p>
		{/if}
		{#if form?.error}
			<p class="text-center text-sm text-red-600">{form.error}</p>
		{/if}
		{#if connectionLost}
			<p class="text-center text-sm text-red-600">Couldn't connect to this chat — try reloading the page.</p>
		{/if}
	</div>

	<form method="POST" use:enhance={() => {
		return async ({ update }) => {
			await update({ reset: true });
		};
	}} class="border-t border-neutral-200 bg-white px-6 py-4">
		<div class="flex items-center gap-2 rounded-full border border-neutral-300 px-4 py-2">
			<input
				name="text"
				type="text"
				placeholder="Ask me anything..."
				required
				class="flex-1 border-none bg-transparent outline-none"
			/>
			<button
				type="submit"
				class="rounded-full bg-neutral-900 px-4 py-1.5 text-sm font-medium text-white hover:bg-neutral-800"
			>
				Send
			</button>
		</div>
	</form>
</div>
