<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { page } from '$app/state';
	import { onMount } from 'svelte';
	import MessageBubble from '$lib/components/MessageBubble.svelte';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Icon from '$lib/components/m3/Icon.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import Select from '$lib/components/m3/Select.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let pendingReply = $state(false);
	let turnError = $state(false);
	let connectionLost = $state(false);
	let modelMenuOpen = $state(false);
	let personalityMenuOpen = $state(false);
	let showCustomForm = $state(false);
	let activitySheetOpen = $state(false);
	let messagesContainer: HTMLDivElement | undefined = $state();
	let messageInput: HTMLInputElement | undefined = $state();

	const TERMINAL_CLOSE_CODES = new Set([4401, 4404]);
	const INITIAL_RETRY_DELAY_MS = 1000;
	const MAX_RETRY_DELAY_MS = 30000;

	const CUSTOM_PROVIDER_OPTIONS = [
		{ value: 'anthropic', label: 'Anthropic' },
		{ value: 'openai', label: 'OpenAI' },
		{ value: 'gemini', label: 'Gemini' },
		{ value: 'fake', label: 'Fake (testing)' },
	];

	const activeModelLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return 'Default model';
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? 'Default model';
		}
		return selection.label;
	});

	const currentPersonalityLabel = $derived.by(() => {
		const current = data.personality.versions.find((v) => v.is_current);
		return current?.description ?? 'Not set';
	});

	const activeDelegationCount = $derived(
		data.agentActivity.filter((d: { status: string }) => d.status === 'pending' || d.status === 'processing').length,
	);

	// Always keep the latest message (and the "Typing…" indicator) in view — re-runs whenever
	// the message list changes or a reply starts streaming.
	$effect(() => {
		data.messages.length;
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
				} else if (envelope.kind === 'AgentDelegationUpdated') {
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
	<div bind:this={messagesContainer} class="flex-1 space-y-4 overflow-y-auto px-6 py-6">
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
		<div class="mt-2 flex items-center gap-1">
			<Menu bind:open={modelMenuOpen}>
				{#snippet trigger({ toggle })}
					<IconButton onclick={toggle} aria-label="Model: {activeModelLabel}">
						<Icon name="agents" size={18} />
					</IconButton>
				{/snippet}
				<div class="w-80">
					<p class="md-label-medium px-2 pt-1 pb-2" style="color: var(--md-sys-color-on-surface-variant)">
						Model — {activeModelLabel}
					</p>
					{#if form?.modelError}
						<p class="md-body-small mb-2 px-2" style="color: var(--md-sys-color-error)">{form.modelError}</p>
					{/if}
					{#each data.models.admin_models as model (model.id)}
						<form
							method="POST"
							action="?/selectAdminModel"
							use:enhance={() => {
								return async ({ update }) => {
									await update();
									modelMenuOpen = false;
								};
							}}
						>
							<input type="hidden" name="admin_model_id" value={model.id} />
							<MenuItem
								type="submit"
								selected={data.models.selection?.kind === 'admin' &&
									data.models.selection.admin_model_id === model.id}
							>
								{model.label}
							</MenuItem>
						</form>
					{/each}

					<p class="md-label-medium mt-3 mb-1 px-2" style="color: var(--md-sys-color-on-surface-variant)">
						Your own key
					</p>
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
									await update({ reset: true });
									showCustomForm = false;
								};
							}}
							class="mt-1 space-y-1 px-2"
						>
							<input name="label" type="text" placeholder="Label" required class="m3-picker-input" />
							<Select label="Provider" name="provider" options={CUSTOM_PROVIDER_OPTIONS} />
							<input name="model_id" type="text" placeholder="Model ID" class="m3-picker-input" />
							<input name="api_key" type="password" placeholder="API key" class="m3-picker-input" />
							<input name="base_url" type="text" placeholder="Base URL (optional)" class="m3-picker-input" />
							<Button type="submit" variant="filled" class="w-full">Save & validate</Button>
						</form>
					{:else}
						<MenuItem type="button" onclick={() => (showCustomForm = true)}>+ Use your own API key</MenuItem>
					{/if}
				</div>
			</Menu>
			<Menu bind:open={personalityMenuOpen}>
				{#snippet trigger({ toggle })}
					<IconButton onclick={toggle} aria-label="Personality: {currentPersonalityLabel}">
						<Icon name="person" size={18} />
					</IconButton>
				{/snippet}
				<div class="w-80">
					<p class="md-label-medium px-2 pt-1 pb-2" style="color: var(--md-sys-color-on-surface-variant)">
						Personality — {currentPersonalityLabel}
					</p>
					{#if form?.personalityError}
						<p class="md-body-small mb-2 px-2" style="color: var(--md-sys-color-error)">{form.personalityError}</p>
					{/if}
					{#if data.personality.versions.length === 0}
						<p class="md-body-medium px-2 py-1" style="color: var(--md-sys-color-on-surface-variant)">
							You haven't set a personality yet — just ask nomi to change it.
						</p>
					{:else}
						<List>
							{#each data.personality.versions as version (version.version)}
								<ListItem
									headline={version.description}
									supportingText={`v${version.version} · ${new Date(version.created_at).toLocaleString()}`}
									selected={version.is_current}
								>
									{#snippet trailing()}
										{#if !version.is_current}
											<form
												method="POST"
												action="?/restorePersonality"
												use:enhance={() => {
													return async ({ update }) => {
														await update();
														personalityMenuOpen = false;
													};
												}}
											>
												<input type="hidden" name="version" value={version.version} />
												<Button type="submit" variant="text">Restore</Button>
											</form>
										{/if}
									{/snippet}
								</ListItem>
							{/each}
						</List>
					{/if}
				</div>
			</Menu>
		</div>
	</div>
</div>

<BottomSheet bind:open={activitySheetOpen}>
	{#snippet children()}
		<h2 class="md-title-large" style="color: var(--md-sys-color-on-surface); margin: 0 0 12px;">Agent activity</h2>
		{#if data.agentActivity.length === 0}
			<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">No background activity yet.</p>
		{:else}
			{#each data.agentActivity as item (item.id)}
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

<style>
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
