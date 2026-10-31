<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import { mobileNav } from '$lib/mobileNav.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconPerson from '$lib/components/icons/IconPerson.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import ModelManager from '$lib/components/models/ModelManager.svelte';
	import ModelSelector from '$lib/components/models/ModelSelector.svelte';
	import { applyChanges, choiceFor, type ModelChange, type ModelChoice } from '$lib/modelPicker';
	import { postAction } from '$lib/providerActions';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	let personalityMenuOpen = $state(false);

	// The composer's model picker. Both follow the page's data, and are moved ahead of the server
	// while a change is on its way (a pick that fails is put back).
	let modelGroups = $derived(data.modelOptions.groups);
	let sessionModel = $derived(data.sessionModel);
	let manageOpen = $state(false);
	let modelErrorOpen = $state(false);

	async function pickModel(key: string) {
		const previous = sessionModel;
		sessionModel = choiceFor(modelGroups, key) ?? previous;
		const body = new FormData();
		body.set('key', key);
		const outcome = await postAction<{ sessionModel: ModelChoice }>('setSessionModel', body);
		if (outcome.ok) {
			sessionModel = outcome.data.sessionModel;
		} else {
			sessionModel = previous;
			modelErrorOpen = true;
		}
	}

	const modelsSaved = (changes: ModelChange[]) => (modelGroups = applyChanges(modelGroups, changes));

	// What this chat replies in: its own personality, else the person's default.
	const chatOwn = $derived(data.chatPersonality.chat);
	const currentPersonalityLabel = $derived(chatOwn ?? data.chatPersonality.default ?? m.chatpage_not_set());
	const closeAfter = () => async ({ update }: { update: () => Promise<void> }) => {
		await update();
		personalityMenuOpen = false;
	};
</script>

<ChatThread
	onopenmenu={() => (mobileNav.open = true)}
	sessionId={page.params.sessionId as string}
	title={data.title}
	messages={data.messages}
	hasOlder={data.hasOlder}
	agentActivity={data.agentActivity}
	agentStatus={data.agentStatus}
	sendError={form?.error ?? null}
	glass={page.data.preferences?.animated_backdrop === true}
	thinkingLevel={data.thinkingLevel}
	chatMenu={data.chatMenu}
>
	{#snippet modelControl()}
		{#if modelGroups.length > 0 || sessionModel}
			<ModelSelector groups={modelGroups} selected={sessionModel} onselect={pickModel} onmanage={() => (manageOpen = true)} />
		{/if}
	{/snippet}
	{#snippet extraControls()}
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

<!-- Outside the thread: its composer is a form, and these hold forms of their own. -->
<ModelManager bind:open={manageOpen} groups={modelGroups} onsaved={modelsSaved} />
<Snackbar bind:open={modelErrorOpen} message={m.msel_pick_failed()} />

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
