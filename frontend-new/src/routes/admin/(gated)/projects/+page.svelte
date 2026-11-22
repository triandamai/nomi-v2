<script lang="ts">
	import { FolderKanban, SearchX } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AdminProject } from '$lib/admin/api';
	import { dateOnly } from '$lib/admin/format';
	import { issueSummary, sandboxSummary } from '$lib/admin/projects';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import FilterBar from '$lib/ds/admin/FilterBar.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Projects (projects: view): every project with its owner and workspace, its sandbox (how many it
	// has had, and whether one is active) and its issues; each count opens the page about it.
	let { data } = $props();

	const language = getLocale();
	const link = 'rounded-xs text-text outline-offset-2 hover:underline';

	const columns: TableColumn<AdminProject>[] = [
		{ key: 'project', label: m.admin_col_project(), sort: (row) => row.name },
		{ key: 'owner', label: m.admin_projects_col_owner(), phone: false, sort: (row) => row.owner_email ?? '' },
		{ key: 'sandbox', label: m.admin_projects_col_sandbox() },
		{ key: 'issues', label: m.admin_projects_col_issues(), sort: (row) => row.open_issues },
		{ key: 'created', label: m.admin_projects_col_created(), phone: false, sort: (row) => row.created_at },
	];
</script>

<PageHeader title={m.admin_page_projects()} intro={m.admin_projects_intro()} />

<FilterBar label={m.admin_projects_search()} placeholder={m.admin_projects_search_placeholder()} query={data.query} submitLabel={m.admin_search_submit()} />

{#if data.projects.ok}
	{#if data.projects.data.length === 0}
		{#if data.query}
			<EmptyState icon={SearchX} title={m.admin_projects_none_title({ query: data.query })} body={m.admin_projects_none_body()} />
		{:else}
			<EmptyState icon={FolderKanban} title={m.admin_projects_empty_title()} body={m.admin_projects_empty_body()} />
		{/if}
	{:else}
		<AdminTable label={m.admin_projects_table()} {columns} rows={data.projects.data} rowKey={(row) => row.id}>
			{#snippet cell(row, column)}
				{#if column.key === 'project'}
					<span class="flex min-w-0 flex-col">
						<span class="font-mono font-medium text-text">{row.name}</span>
						{#if row.workspace}<span class="text-caption text-text-3">{row.workspace}</span>{/if}
					</span>
				{:else if column.key === 'owner'}
					<span class="text-text-2">{row.owner_email ?? '—'}</span>
				{:else if column.key === 'sandbox'}
					<a href="/admin/sandboxes?project={row.id}" class={link} aria-label={m.admin_projects_sandbox_link({ project: row.name, summary: sandboxSummary(row) })}>{sandboxSummary(row)}</a>
				{:else if column.key === 'issues'}
					<a href="/admin/projects/{row.id}/issues" class={link} aria-label={m.admin_projects_issues_link({ project: row.name, summary: issueSummary(row) })}>{issueSummary(row)}</a>
				{:else if column.key === 'created'}
					<span class="font-mono text-label text-text-2">{dateOnly(row.created_at, language)}</span>
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_projects_failed()} problem={data.projects.problem} />
{/if}
