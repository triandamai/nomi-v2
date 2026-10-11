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
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import ListToolbar from '$lib/components/ListToolbar.svelte';
	import { onMount } from 'svelte';
	import { timeAgo } from '$lib/i18n';
	import { pageCount, pageOf, searchItems, storedLayout, storeLayout, type ListLayout } from '$lib/listView';
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

	const PAGE_SIZE = 12;
	let query = $state('');
	let page = $state(1);
	let layout = $state<ListLayout>('table');
	onMount(() => (layout = storedLayout('projects', 'table')));
	const found = $derived(searchItems(data.projects, query, (p) => `${p.name} ${p.description ?? ''} ${p.created_by}`));
	const pages = $derived(pageCount(found.length, PAGE_SIZE));
	const shown = $derived(pageOf(found, page, PAGE_SIZE));
	// A new search, or another organization, starts from the first page.
	$effect(() => {
		void query;
		void data.orgId;
		page = 1;
	});

	const columns = [
		{ key: 'name', label: m.list_col_project() },
		{ key: 'status', label: m.list_col_status() },
		{ key: 'by', label: m.list_col_started_by() },
		{ key: 'issues', label: m.list_col_open_issues() },
		{ key: 'updated', label: m.list_col_updated() },
	];

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
		<div class="list-bar">
			<ListToolbar bind:query bind:layout placeholder={m.projects_search()} onlayout={(next) => storeLayout('projects', next)} />
			<form method="POST" action="?/createProject" use:enhance>
				<input type="hidden" name="org_id" value={currentOrg.id} />
				<Button type="submit" variant="filled" size="m"><IconPlus size={20} />{m.projects_add()}</Button>
			</form>
		</div>

		{#if data.projects.length === 0}
			<p class="projects__empty">{m.projects_empty()}</p>
		{:else if found.length === 0}
			<p class="projects__empty">{m.list_no_match({ query })}</p>
		{:else if layout === 'table'}
			<div class="table-wrap">
			<DataTable card {columns}>
				{#each shown as project (project.id)}
					<tr>
						<td>
							<a class="cell-name" href="/projects/session/{project.session_id}" data-sveltekit-reload>
								<span class="cell-name__title">{project.name}</span>
								{#if project.description}<span class="cell-name__description">{project.description}</span>{/if}
							</a>
						</td>
						<td><span class="status status--{project.status}">{STATUS_LABEL[project.status] ?? project.status}</span></td>
						<td class="muted">{project.created_by}</td>
						<td class="muted numeric">
							{#if project.open_issues > 0}<a class="issues-link" href="/projects/{project.id}/issues">{project.open_issues}</a>{:else}0{/if}
						</td>
						<td class="muted numeric">{timeAgo(project.updated_at)}</td>
					</tr>
				{/each}
				{#snippet list()}
					{#each shown as project (project.id)}
						<a role="listitem" class="row-item" href="/projects/session/{project.session_id}" data-sveltekit-reload>
							<span class="row-item__top">
								<span class="row-item__title">{project.name}</span>
								<span class="status status--{project.status}">{STATUS_LABEL[project.status] ?? project.status}</span>
							</span>
							<span class="row-item__meta">
								{m.projects_by({ name: project.created_by })} · {timeAgo(project.updated_at)}{#if project.open_issues > 0} · {issuesLabel(project.open_issues)}{/if}
							</span>
						</a>
					{/each}
				{/snippet}
			</DataTable>
			</div>
		{:else}
			<ul class="grid">
				{#each shown as project (project.id)}
					<li class="card">
						<a class="card__link" href="/projects/session/{project.session_id}" data-sveltekit-reload>
							<span class="card__name">{project.name}</span>
						</a>
						<p class="card__description" class:card__description--empty={!project.description}>{project.description ?? m.projects_no_description()}</p>
						<p class="card__meta">
							<span class="card__status card__status--{project.status}">{STATUS_LABEL[project.status] ?? project.status}</span>
							<span>{m.projects_by({ name: project.created_by })} · {timeAgo(project.updated_at)}</span>
							{#if project.open_issues > 0}<span class="card__issues">{issuesLabel(project.open_issues)}</span>{/if}
						</p>
					</li>
				{/each}
			</ul>
		{/if}

		{#if pages > 1}
			<div class="pager">
				<Pagination {page} {pages} onselect={(next) => (page = next)} label={m.page_pages()} />
			</div>
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
	.list-bar {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
		margin-top: 24px;
	}
	.list-bar > :global(:first-child) {
		flex: 1 1 320px;
	}
	.pager {
		display: flex;
		justify-content: center;
		margin-top: 20px;
	}
	.table-wrap {
		margin-top: 16px;
	}
	.cell-name {
		display: flex;
		flex-direction: column;
		gap: 2px;
		max-width: 360px;
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.cell-name__title {
		overflow: hidden;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.cell-name:hover .cell-name__title {
		text-decoration: underline;
		text-underline-offset: 3px;
	}
	.cell-name__description {
		overflow: hidden;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.muted {
		color: var(--md-sys-color-on-surface-variant);
	}
	.numeric {
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.issues-link {
		color: var(--md-sys-color-primary);
		font-weight: 600;
	}
	.status {
		display: inline-block;
		padding: 2px 8px;
		border-radius: 4px;
		font-size: 0.75rem;
		font-weight: 600;
		white-space: nowrap;
		background: var(--md-sys-color-surface-container-highest);
		color: var(--md-sys-color-on-surface-variant);
	}
	.status--building {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.status--ready {
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.row-item {
		display: flex;
		flex-direction: column;
		gap: 4px;
		padding: 12px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
	}
	.row-item__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 8px;
	}
	.row-item__title {
		overflow: hidden;
		font-weight: 600;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.row-item__meta {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
		gap: 12px;
		margin: 16px 0 0;
		padding: 0;
		list-style: none;
	}
	.card {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 6px;
		min-height: 156px;
		padding: 16px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface);
		transition: background-color 150ms ease-out;
	}
	.card:hover {
		background: var(--md-sys-color-surface-container-low);
	}
	.card:has(.card__link:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.card__link {
		color: var(--md-sys-color-on-surface);
		text-decoration: none;
		outline: none;
	}
	.card__link::after {
		content: '';
		position: absolute;
		inset: 0;
		border-radius: inherit;
	}
	.card__name {
		display: -webkit-box;
		overflow: hidden;
		font-size: 1.0625rem;
		font-weight: 650;
		line-height: 1.3;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		-webkit-box-orient: vertical;
	}
	.card__description {
		display: -webkit-box;
		overflow: hidden;
		margin: 0;
		font-size: 0.875rem;
		line-height: 1.45;
		color: var(--md-sys-color-on-surface-variant);
		-webkit-line-clamp: 3;
		line-clamp: 3;
		-webkit-box-orient: vertical;
	}
	.card__description--empty {
		font-style: italic;
	}
	.card__meta {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 4px 12px;
		margin: auto 0 0;
		padding-top: 8px;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	/* Status as plain text: only "Ready" takes the accent. */
	.card__status {
		font-weight: 600;
		color: var(--md-sys-color-on-surface);
	}
	.card__status--ready {
		color: var(--md-sys-color-primary);
	}
	.card__issues {
		margin-left: auto;
		font-weight: 600;
		color: var(--md-sys-color-primary);
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
