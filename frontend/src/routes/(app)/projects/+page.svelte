<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { enhance } from '$app/forms';
	import Dialog from '$lib/components/m3/Dialog.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Menu from '$lib/components/m3/Menu.svelte';
	import MenuItem from '$lib/components/m3/MenuItem.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import OrgPicker from '$lib/components/OrgPicker.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconChevronDown from '$lib/components/icons/IconChevronDown.svelte';
	import IconCheck from '$lib/components/icons/IconCheck.svelte';
	import IconAgents from '$lib/components/icons/IconAgents.svelte';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	const STATUS_LABEL: Record<string, string> = {
		planning: m.projects_planning(),
		building: m.projects_building(),
		ready: m.projects_ready(),
	};

	const currentOrg = $derived(data.orgs.find((org) => org.id === data.orgId) ?? null);
	const orgError = $derived(form && 'orgError' in form ? (form.orgError as string) : null);

	let switcherOpen = $state(false);
	let newOrgOpen = $state(false);
	let newOrgName = $state('');
	let switchForm: HTMLFormElement | undefined = $state();
	let switchTo = $state('');

	function switchOrg(orgId: string) {
		switcherOpen = false;
		if (orgId === data.orgId) return;
		switchTo = orgId;
		queueMicrotask(() => switchForm?.requestSubmit());
	}

	function issuesLabel(count: number) {
		return count === 1 ? m.projects_open_issue() : m.projects_open_issues({ count });
	}
</script>

<div class="projects">
	<header class="projects__head">
		<div>
			<h1 class="md-display-small projects__title">{m.projects_title()}</h1>
			<p class="md-body-large projects__lede">{m.projects_lede()}</p>
		</div>
		{#if currentOrg}
			<div class="projects__org">
				<Menu bind:open={switcherOpen}>
					{#snippet trigger({ toggle })}
						<button type="button" class="org-switch" onclick={toggle} aria-haspopup="menu" aria-expanded={switcherOpen} aria-label={m.projects_switch_org()}>
							<span class="org-switch__mark" aria-hidden="true">{currentOrg.name.trim().charAt(0).toUpperCase()}</span>
							<span class="org-switch__name">{currentOrg.name}</span>
							<IconChevronDown size={18} />
						</button>
					{/snippet}
					{#each data.orgs as org (org.id)}
						<MenuItem selected={org.id === data.orgId} onclick={() => switchOrg(org.id)}>
							<span class="menu-row">
								<span class="menu-row__name">{org.name}</span>
								{#if org.id === data.orgId}<IconCheck size={18} />{/if}
							</span>
						</MenuItem>
					{/each}
					<MenuItem onclick={() => { switcherOpen = false; newOrgOpen = true; }}>
						<span class="menu-row"><span class="menu-row__name">{m.projects_new_org()}</span><IconPlus size={18} /></span>
					</MenuItem>
				</Menu>
				<Button variant="text" size="s" href="/organizations/{currentOrg.id}">
					<IconAgents size={18} />
					{m.projects_manage_team()}
				</Button>
			</div>
			<form method="POST" action="?/chooseOrg" bind:this={switchForm} use:enhance hidden>
				<input type="hidden" name="org_id" value={switchTo} />
			</form>
		{/if}
	</header>

	{#if !currentOrg || (form && 'needsOrg' in form)}
		<section class="pick" aria-labelledby="pick-title">
			<h2 id="pick-title" class="pick__title">{m.org_pick_title()}</h2>
			<p class="pick__lede">{m.org_pick_lede()}</p>
			<OrgPicker orgs={data.orgs} current={data.orgId} error={orgError} />
		</section>
	{:else}
		<div class="grid">
			<form method="POST" action="?/createProject" use:enhance>
				<input type="hidden" name="org_id" value={currentOrg.id} />
				<button type="submit" class="add-tile">
					<IconPlus size={28} />
					<span class="md-title-medium">{m.projects_add()}</span>
				</button>
			</form>

			{#each data.projects as project (project.id)}
				<a href="/projects/session/{project.session_id}" class="card" data-sveltekit-reload>
					<div class="card__top">
						<p class="card__name">{project.name}</p>
						<span class="card__status card__status--{project.status}">{STATUS_LABEL[project.status] ?? project.status}</span>
					</div>
					{#if project.description}
						<p class="card__description">{project.description}</p>
					{/if}
					<p class="card__meta">
						<span>{m.projects_by({ name: project.created_by })}</span>
						{#if project.open_issues > 0}<span>{issuesLabel(project.open_issues)}</span>{/if}
					</p>
				</a>
			{/each}
		</div>
		{#if data.projects.length === 0}
			<p class="projects__empty">{m.projects_empty()}</p>
		{/if}
	{/if}
</div>

<Dialog bind:open={newOrgOpen} headline={m.org_new_title()}>
	<form
		id="new-org"
		method="POST"
		action="?/createOrg"
		use:enhance={() => async ({ update, result }) => {
			await update();
			if (result.type === 'success') {
				newOrgOpen = false;
				newOrgName = '';
			}
		}}
	>
		<p class="dialog-lede">{m.org_pick_new_hint()}</p>
		<TextField name="name" label={m.org_name()} bind:value={newOrgName} maxlength={80} required />
		{#if orgError && newOrgOpen}<p class="dialog-error" role="alert">{orgError}</p>{/if}
	</form>
	{#snippet actions()}
		<Button variant="text" type="button" onclick={() => (newOrgOpen = false)}>{m.org_cancel()}</Button>
		<Button variant="filled" type="submit" form="new-org" disabled={!newOrgName.trim()}>{m.org_create()}</Button>
	{/snippet}
</Dialog>

<style>
	.projects {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 40px) 56px;
		box-sizing: border-box;
	}
	.projects__head {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-end;
		justify-content: space-between;
		gap: 16px;
	}
	.projects__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.projects__lede {
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.projects__org {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px;
	}
	.org-switch {
		display: inline-flex;
		align-items: center;
		gap: 10px;
		max-width: min(320px, 100%);
		min-height: 44px;
		padding: 4px 12px 4px 4px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: var(--md-sys-color-surface);
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-weight: 600;
		cursor: pointer;
	}
	.org-switch:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.org-switch:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.org-switch__mark {
		display: grid;
		flex: none;
		place-items: center;
		width: 34px;
		height: 34px;
		border-radius: 6px;
		background: var(--md-sys-color-primary);
		color: var(--md-sys-color-on-primary);
	}
	.org-switch__name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.menu-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		min-width: 200px;
	}
	.menu-row__name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.pick {
		max-width: 560px;
		margin-top: 32px;
	}
	.pick__title {
		margin: 0;
		font-family: var(--md-sys-typescale-headline-small-font);
		font-size: var(--md-sys-typescale-headline-small-size);
		line-height: 1.2;
		color: var(--md-sys-color-on-surface);
	}
	.pick__lede {
		margin: 8px 0 20px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 12px;
		margin-top: 24px;
	}
	.add-tile {
		display: flex;
		height: 100%;
		width: 100%;
		min-height: 112px;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		border-radius: var(--md-sys-shape-corner-medium);
		border: 1px dashed var(--md-sys-color-outline);
		background: none;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
	}
	.add-tile:hover {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
	}
	.add-tile:focus-visible,
	.card:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 16px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.card:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.card__top {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 8px;
	}
	.card__name {
		margin: 0;
		min-width: 0;
		overflow: hidden;
		font-weight: 700;
		font-size: 1.0625rem;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.card__status {
		flex: none;
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__status--ready {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.card__description {
		margin: 0;
		display: -webkit-box;
		overflow: hidden;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
		font-size: 0.9375rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card__meta {
		display: flex;
		flex-wrap: wrap;
		gap: 4px 12px;
		margin: auto 0 0;
		padding-top: 6px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.projects__empty {
		margin: 16px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.dialog-lede {
		margin: 0 0 16px;
		color: var(--md-sys-color-on-surface-variant);
	}
	.dialog-error {
		margin: 8px 0 0;
		color: var(--md-sys-color-error);
	}
</style>
