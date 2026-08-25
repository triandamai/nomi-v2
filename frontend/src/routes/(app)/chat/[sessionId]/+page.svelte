<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import Button from '$lib/components/m3/Button.svelte';
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

<div class="flex h-full flex-col" style="background: var(--md-sys-color-surface)">
	<div class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
		{#each data.messages as message (message.id)}
			<MessageBubble {message} />
		{/each}
		{#if pendingReply}
			<div class="flex justify-start">
				<div
					class="md-body-large max-w-md px-4 py-2"
					style="background: var(--md-sys-color-surface-container-high); color: var(--md-sys-color-on-surface-variant); border-radius: var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-large) var(--md-sys-shape-corner-extra-small)"
				>
					Typing…
				</div>
			</div>
		{/if}
		{#if turnError}
			<p class="md-body-medium text-center" style="color: var(--md-sys-color-error)">
				Something went wrong — try sending again.
			</p>
		{/if}
		{#if form?.error}
			<p class="md-body-medium text-center" style="color: var(--md-sys-color-error)">{form.error}</p>
		{/if}
		{#if connectionLost}
			<p class="md-body-medium text-center" style="color: var(--md-sys-color-error)">
				Couldn't connect to this chat — try reloading the page.
			</p>
		{/if}
	</div>

	<div
		class="px-6 py-4"
		style="background: var(--md-sys-color-surface-container-low); border-top: 1px solid var(--md-sys-color-outline-variant)"
	>
		<div class="mb-2 flex justify-end">
			<div class="relative">
				<Button type="button" variant="outlined" onclick={() => (modelPickerOpen = !modelPickerOpen)}>
					{activeModelLabel}
				</Button>
				{#if modelPickerOpen}
					<div
						class="absolute right-0 bottom-full mb-2 w-72 p-3"
						style="background: var(--md-sys-color-surface-container-high); border-radius: var(--md-sys-shape-corner-extra-large); box-shadow: var(--md-sys-elevation-shadow-level3)"
					>
						{#if form?.modelError}
							<p class="md-body-small mb-2" style="color: var(--md-sys-color-error)">{form.modelError}</p>
						{/if}
						<p class="md-label-medium mb-1" style="color: var(--md-sys-color-on-surface-variant)">Available models</p>
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
									class="m3-picker-item"
									class:m3-picker-item--selected={data.models.selection?.kind === 'admin' &&
										data.models.selection.admin_model_id === model.id}
								>
									{model.label}
								</button>
							</form>
						{/each}

						<p class="md-label-medium mt-3 mb-1" style="color: var(--md-sys-color-on-surface-variant)">Your own key</p>
						{#if data.models.selection?.kind === 'custom'}
							<p class="md-body-medium px-2 py-1" style="font-weight: 600; color: var(--md-sys-color-on-surface)">
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
								<input name="label" type="text" placeholder="Label" required class="m3-picker-input" />
								<select name="provider" required class="m3-picker-input">
									<option value="anthropic">Anthropic</option>
									<option value="openai">OpenAI</option>
									<option value="gemini">Gemini</option>
									<option value="fake">Fake (testing)</option>
								</select>
								<input name="model_id" type="text" placeholder="Model ID" class="m3-picker-input" />
								<input name="api_key" type="password" placeholder="API key" class="m3-picker-input" />
								<input name="base_url" type="text" placeholder="Base URL (optional)" class="m3-picker-input" />
								<Button type="submit" variant="filled" class="w-full">Save & validate</Button>
							</form>
						{:else}
							<button type="button" onclick={() => (showCustomForm = true)} class="m3-picker-item mt-1">
								+ Use your own API key
							</button>
						{/if}
					</div>
				{/if}
			</div>
		</div>
		<form
			method="POST"
			action="?/sendMessage"
			use:enhance={() => {
				return async ({ update }) => {
					await update({ reset: true });
				};
			}}
		>
			<div
				class="flex items-center gap-2 px-4 py-2"
				style="background: var(--md-sys-color-surface); border-radius: var(--md-sys-shape-corner-full); border: 1px solid var(--md-sys-color-outline)"
			>
				<input
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
	</div>
</div>

<style>
	.m3-picker-item {
		display: block;
		width: 100%;
		border: none;
		background: transparent;
		cursor: pointer;
		text-align: left;
		padding: 8px;
		border-radius: var(--md-sys-shape-corner-small);
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
		color: var(--md-sys-color-on-surface);
	}
	.m3-picker-item:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-picker-item--selected {
		font-weight: 600;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}

	.m3-picker-input {
		width: 100%;
		box-sizing: border-box;
		border-radius: var(--md-sys-shape-corner-small);
		border: 1px solid var(--md-sys-color-outline);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		padding: 6px 8px;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.m3-picker-input:focus {
		outline: none;
		border: 2px solid var(--md-sys-color-primary);
		padding: 5px 7px;
	}
</style>
