<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconAgents from '$lib/components/icons/IconAgents.svelte';
	import IconPerson from '$lib/components/icons/IconPerson.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import ModelKeySheet from '$lib/components/ModelKeySheet.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let modelMenuOpen = $state(false);
	let personalityMenuOpen = $state(false);
	let modelSheetOpen = $state(false);

	const activeModelLabel = $derived.by(() => {
		const selection = data.models.selection;
		if (!selection) return m.models_default();
		if (selection.kind === 'admin') {
			const match = data.models.admin_models.find((m) => m.id === selection.admin_model_id);
			return match?.label ?? m.models_default();
		}
		return selection.label;
	});

	// What this chat replies in: its own personality, else the person's default.
	const chatOwn = $derived(data.chatPersonality.chat);
	const currentPersonalityLabel = $derived(chatOwn ?? data.chatPersonality.default ?? m.chatpage_not_set());
	const closeAfter = () => async ({ update }: { update: () => Promise<void> }) => {
		await update();
		personalityMenuOpen = false;
	};
</script>

<ChatThread
	sessionId={page.params.sessionId as string}
	title={data.title}
	messages={data.messages}
	hasOlder={data.hasOlder}
	agentActivity={data.agentActivity}
	agentStatus={data.agentStatus}
	sendError={form?.error ?? null}
	glass={page.data.preferences?.animated_backdrop !== false}
	thinkingLevel={data.thinkingLevel}
>
	{#snippet extraControls()}
		<Menu bind:open={modelMenuOpen}>
			{#snippet trigger({ toggle })}
				<IconButton onclick={toggle} aria-label={m.chatpage_model_label({ model: activeModelLabel })}>
					<IconAgents size={18} />
				</IconButton>
			{/snippet}
			<div class="w-full">
				<p class="md-label-medium px-2 pt-1 pb-2" style="color: var(--md-sys-color-on-surface-variant)">
					{m.chatpage_model_heading({ model: activeModelLabel })}
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
					{m.chatpage_own_key()}
				</p>
				{#if data.models.selection?.kind === 'custom'}
					<p class="md-body-medium px-2 py-1" style="font-weight: 600; color: var(--md-sys-color-on-surface)">
						{data.models.selection.label} ({data.models.selection.api_key_masked})
					</p>
				{/if}
				<MenuItem
					type="button"
					onclick={() => {
						modelMenuOpen = false;
						modelSheetOpen = true;
					}}
				>
					{data.models.selection?.kind === 'custom' ? m.models_change_key() : m.chatpage_use_own_key()}
				</MenuItem>
			</div>
		</Menu>
		<ModelKeySheet bind:open={modelSheetOpen} error={form?.modelError ?? null} />
		<Menu bind:open={personalityMenuOpen}>
			{#snippet trigger({ toggle })}
				<IconButton onclick={toggle} aria-label={m.chatpage_personality_label({ personality: currentPersonalityLabel })}>
					<IconPerson size={18} />
				</IconButton>
			{/snippet}
			<div class="w-full">
				<div class="persona-now">
					<span class="persona-now__label">{chatOwn ? m.chatpage_personality_this_chat() : m.chatpage_personality_default()}</span>
					<span class="persona-now__text">{currentPersonalityLabel}</span>
					{#if chatOwn}
						<form method="POST" action="?/resetPersonality" use:enhance={closeAfter}>
							<Button type="submit" variant="text">{m.chatpage_personality_reset()}</Button>
						</form>
					{/if}
					<span class="persona-now__hint">{m.chatpage_personality_hint()}</span>
				</div>
				{#if form?.personalityError}
					<p class="md-body-small mb-2 px-2" style="color: var(--md-sys-color-error)">{form.personalityError}</p>
				{/if}
				{#if data.personality.versions.length === 0}
					<p class="md-body-medium px-2 py-1" style="color: var(--md-sys-color-on-surface-variant)">
						{m.chatpage_no_personality()}
					</p>
				{:else}
					<p class="md-label-medium px-2 pt-2" style="color: var(--md-sys-color-on-surface-variant)">{m.chatpage_personality_history()}</p>
					<ul class="persona-list">
						{#each data.personality.versions as version (version.version)}
							<li class="persona-row" class:persona-row--active={version.description === currentPersonalityLabel}>
								<span class="persona-row__text">{version.description}</span>
								<span class="persona-row__meta">v{version.version} · {new Date(version.created_at).toLocaleDateString(getLocale())}{version.is_current ? ` · ${m.chatpage_personality_is_default()}` : ''}</span>
								<span class="persona-actions">
									{#if version.description !== chatOwn}
										<form method="POST" action="?/usePersonality" use:enhance={closeAfter}>
											<input type="hidden" name="description" value={version.description} />
											<input type="hidden" name="scope" value="chat" />
											<Button type="submit" variant="text">{m.chatpage_use_here()}</Button>
										</form>
									{/if}
									{#if !version.is_current || chatOwn}
										<form method="POST" action="?/usePersonality" use:enhance={closeAfter}>
											<input type="hidden" name="description" value={version.description} />
											<input type="hidden" name="scope" value="all_chats" />
											<Button type="submit" variant="text">{m.chatpage_use_everywhere()}</Button>
										</form>
									{/if}
								</span>
							</li>
						{/each}
					</ul>
				{/if}
			</div>
		</Menu>
	{/snippet}
</ChatThread>

<style>
	.persona-now {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		gap: 2px;
		padding: 4px 8px 8px;
	}
	.persona-now__label {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
		font-weight: 600;
	}
	.persona-now__text {
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.persona-now__hint {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
	.persona-list {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-height: 50vh;
		margin: 4px 0 0;
		padding: 0;
		overflow-y: auto;
		list-style: none;
	}
	.persona-row {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 8px 2px;
		border-radius: var(--md-sys-shape-corner-medium);
	}
	.persona-row--active {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.persona-row__text {
		font-size: 0.875rem;
		overflow-wrap: anywhere;
	}
	.persona-row__meta {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
	.persona-actions {
		display: flex;
		flex-wrap: wrap;
		margin-left: -12px;
	}
</style>
