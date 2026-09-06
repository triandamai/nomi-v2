<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import ChatThread from '$lib/components/ChatThread.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconAgents from '$lib/components/icons/IconAgents.svelte';
	import IconPerson from '$lib/components/icons/IconPerson.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
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
</script>

<ChatThread sessionId={page.params.sessionId as string} messages={data.messages} agentActivity={data.agentActivity} sendError={form?.error ?? null}>
	{#snippet extraControls()}
		<Menu bind:open={modelMenuOpen}>
			{#snippet trigger({ toggle })}
				<IconButton onclick={toggle} aria-label="Model: {activeModelLabel}">
					<IconAgents size={18} />
				</IconButton>
			{/snippet}
			<div class="w-full">
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
				<MenuItem
					type="button"
					onclick={() => {
						modelMenuOpen = false;
						modelSheetOpen = true;
					}}
				>
					{data.models.selection?.kind === 'custom' ? 'Change your key' : '+ Use your own API key'}
				</MenuItem>
			</div>
		</Menu>
		<ModelKeySheet bind:open={modelSheetOpen} error={form?.modelError ?? null} />
		<Menu bind:open={personalityMenuOpen}>
			{#snippet trigger({ toggle })}
				<IconButton onclick={toggle} aria-label="Personality: {currentPersonalityLabel}">
					<IconPerson size={18} />
				</IconButton>
			{/snippet}
			<div class="w-full">
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
	{/snippet}
</ChatThread>
