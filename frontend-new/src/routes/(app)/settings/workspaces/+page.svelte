<script lang="ts">
	import { invalidate, invalidateAll } from '$app/navigation';
	import { m } from '$lib/paraglide/messages';
	import { pageAction } from '$lib/pageAction';
	import { workspaceCards } from '$lib/settings/workspaces';
	import { toasts } from '$lib/ds/overlays/toasts.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Dialog from '$lib/ds/overlays/Dialog.svelte';
	import Select from '$lib/ds/inputs/Select.svelte';
	import TextField from '$lib/ds/inputs/TextField.svelte';
	import SettingsGroup from '$lib/ds/settings/SettingsGroup.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import WorkspaceCard from '$lib/ds/settings/WorkspaceCard.svelte';
	import CreateWorkspaceCard from '$lib/ds/settings/CreateWorkspaceCard.svelte';

	// Settings › Workspaces (App / Settings / Workspaces l222Mz): a card per workspace (its role,
	// people and projects, where projects come from or the button to choose it, and its Settings),
	// one to create another, and where new projects go. Left out (AGENTS.md §9, BACKEND_TODO.md):
	// Invitations (the API doesn't list a person's invites), templates, who's been active, the
	// card's menu, and "Open Nomi in" (there's no active workspace to open in).
	let { data } = $props();

	const PATH = '/settings/workspaces';
	const cards = $derived(workspaceCards(data.list));

	let using = $state<string>();
	async function use(id: string, name: string) {
		using = id;
		const result = await pageAction(PATH, 'use', { id });
		using = undefined;
		if (!result.ok) {
			if (result.message) toasts.show({ kind: 'error', title: m.workspace_choose_failed(), detail: result.message });
			return;
		}
		toasts.show({ kind: 'success', title: m.workspace_chosen({ name }) });
		await invalidateAll();
	}

	let creating = $state(false);
	let name = $state('');
	let nameError = $state<string>();
	let saving = $state(false);
	function startCreating() {
		name = '';
		nameError = undefined;
		creating = true;
	}
	async function create() {
		saving = true;
		const result = await pageAction(PATH, 'create', { name });
		saving = false;
		if (!result.ok) return void (nameError = result.message || m.workspace_create_failed());
		creating = false;
		toasts.show({ kind: 'success', title: m.workspace_created({ name: name.trim() }) });
		await invalidate('app:workspaces');
		await invalidate('app:shell');
	}
</script>

<svelte:head><title>{m.settings_workspaces()} · {m.app_name()}</title></svelte:head>

<div class="h-full overflow-y-auto">
	<div class="flex max-w-316 flex-col gap-8 px-4 pt-8 pb-12 lg:px-12">
		<header class="flex max-w-180 flex-col gap-1.5">
			<h1 class="text-display font-medium tracking-display text-text">{m.settings_workspaces()}</h1>
			<p class="text-body text-text-2">{m.workspaces_intro()}</p>
		</header>

		<section aria-labelledby="yours-title" class="flex flex-col gap-3">
			<h2 id="yours-title" class="flex items-end gap-2">
				<span class="text-body font-medium text-text">{m.workspaces_yours()}</span>
				<span class="font-mono text-label text-text-3">{cards.length}</span>
			</h2>
			{#if !cards.length}<p class="text-caption text-text-3">{m.workspaces_none()}</p>{/if}
			<div class="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
				{#each cards as card (card.id)}
					<WorkspaceCard
						name={card.name}
						initial={card.initial}
						tone={card.tone}
						meta={card.meta}
						stats={[
							{ value: card.projects, label: card.projects === 1 ? m.workspaces_stat_projects_one() : m.workspaces_stat_projects() },
							{ value: card.members, label: card.members === 1 ? m.workspaces_stat_members_one() : m.workspaces_stat_members() },
						]}
						current={card.current}
						href={card.href}
						settings={m.workspaces_settings()}
						currentTag={m.workspaces_current()}
						currentNote={m.workspaces_current_note()}
						use={m.workspaces_use()}
						busy={using === card.id}
						onuse={() => use(card.id, card.name)}
					/>
				{/each}
				<CreateWorkspaceCard title={m.workspace_create_title()} body={m.workspaces_create_body()} onclick={startCreating} />
			</div>
		</section>

		{#if cards.length}
			<SettingsGroup id="defaults" title={m.workspaces_defaults()}>
				<SettingRow title={m.workspaces_new_projects()} description={m.workspaces_new_projects_body()}>
					{#snippet control()}
						<div class="w-56 max-w-full">
							<Select
								label={m.workspaces_new_projects()}
								options={cards.map((card) => ({ value: card.id, label: card.name }))}
								value={data.list.project_org_id ?? ''}
								placeholder={m.workspaces_new_projects()}
								onchange={(id) => use(id, cards.find((card) => card.id === id)?.name ?? '')}
							/>
						</div>
					{/snippet}
				</SettingRow>
			</SettingsGroup>
		{/if}
	</div>
</div>

<Dialog bind:open={creating} title={m.workspace_create_title()} description={m.workspace_create_body()}>
	<form
		id="create-workspace"
		class="pt-2"
		onsubmit={(event) => {
			event.preventDefault();
			void create();
		}}
	>
		<TextField label={m.workspace_name()} bind:value={name} error={nameError} required />
	</form>
	{#snippet actions()}
		<Button variant="secondary" label={m.action_cancel()} onclick={() => (creating = false)} />
		<Button variant="inverse" label={m.workspace_create_action()} loading={saving} onclick={create} />
	{/snippet}
</Dialog>
