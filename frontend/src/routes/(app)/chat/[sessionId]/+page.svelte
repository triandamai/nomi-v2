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
	let modelPickerOpen = $state(false);
	let showCustomForm = $state(false);

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	const activeModelLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return 'Default model';
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? 'Default model';
		}
		return selection.label;
	});

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

	<div class="border-t border-neutral-200 bg-white px-6 py-4">
		<div class="mb-2 flex justify-end">
			<div class="relative">
				<button
					type="button"
					onclick={() => (modelPickerOpen = !modelPickerOpen)}
					class="rounded-full border border-neutral-300 px-3 py-1 text-xs text-neutral-600 hover:bg-neutral-50"
				>
					{activeModelLabel}
				</button>
				{#if modelPickerOpen}
					<div class="absolute bottom-full right-0 mb-2 w-72 rounded-xl border border-neutral-200 bg-white p-3 shadow-lg">
						{#if form?.modelError}
							<p class="mb-2 text-xs text-red-600">{form.modelError}</p>
						{/if}
						<p class="mb-1 text-xs font-medium text-neutral-500">Available models</p>
						{#each data.models.admin_models as model (model.id)}
							<form
								method="POST"
								action="?/selectAdminModel"
								use:enhance={() => {
									return async ({ update }) => {
										await update();
										modelPickerOpen = false;
									};
								}}
							>
								<input type="hidden" name="admin_model_id" value={model.id} />
								<button
									type="submit"
									class="block w-full rounded-lg px-2 py-1 text-left text-sm hover:bg-neutral-100 {data.models
										.selection?.kind === 'admin' && data.models.selection.admin_model_id === model.id
										? 'font-semibold'
										: ''}"
								>
									{model.label}
								</button>
							</form>
						{/each}

						<p class="mt-3 mb-1 text-xs font-medium text-neutral-500">Your own key</p>
						{#if data.models.selection?.kind === 'custom'}
							<p class="px-2 py-1 text-sm font-semibold">
								{data.models.selection.label} ({data.models.selection.api_key_masked})
							</p>
						{/if}
						{#if showCustomForm}
							<form
								method="POST"
								action="?/selectCustomModel"
								use:enhance={() => {
									return async ({ update }) => {
										await update();
										modelPickerOpen = true;
										showCustomForm = false;
									};
								}}
								class="mt-1 space-y-1"
							>
								<input name="label" type="text" placeholder="Label" required class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<select name="provider" required class="w-full rounded border border-neutral-300 px-2 py-1 text-sm">
									<option value="anthropic">Anthropic</option>
									<option value="openai">OpenAI</option>
									<option value="gemini">Gemini</option>
									<option value="fake">Fake (testing)</option>
								</select>
								<input name="model_id" type="text" placeholder="Model ID" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<input name="api_key" type="password" placeholder="API key" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<input name="base_url" type="text" placeholder="Base URL (optional)" class="w-full rounded border border-neutral-300 px-2 py-1 text-sm" />
								<button type="submit" class="w-full rounded bg-neutral-900 px-2 py-1 text-sm text-white">Save & validate</button>
							</form>
						{:else}
							<button
								type="button"
								onclick={() => (showCustomForm = true)}
								class="mt-1 block w-full rounded-lg px-2 py-1 text-left text-sm text-neutral-600 hover:bg-neutral-100"
							>
								+ Use your own API key
							</button>
						{/if}
					</div>
				{/if}
			</div>
		</div>
		<form method="POST" use:enhance={() => {
			return async ({ update }) => {
				await update({ reset: true });
			};
		}}>
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
</div>
