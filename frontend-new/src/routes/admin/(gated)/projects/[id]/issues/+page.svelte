<script lang="ts">
	import { CircleDot } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AdminIssue } from '$lib/admin/api';
	import { dateOnly } from '$lib/admin/format';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Projects → a project's issues (projects: view), read only: what's open and what was closed,
	// for an admin who isn't one of the project's people. They work on them in the project itself.
	let { data } = $props();

	const language = getLocale();
	const back = { href: '/admin/projects', label: m.admin_projects_back() };

	const columns: TableColumn<AdminIssue>[] = [
		{ key: 'number', label: '#', sort: (row) => row.number },
		{ key: 'title', label: m.admin_issues_col_title(), sort: (row) => row.title },
		{ key: 'kind', label: m.admin_issues_col_kind(), phone: false },
		{ key: 'severity', label: m.admin_issues_col_severity(), phone: false },
		{ key: 'status', label: m.admin_col_status() },
		{ key: 'column', label: m.admin_issues_col_column(), phone: false },
		{ key: 'updated', label: m.admin_issues_col_updated(), phone: false, sort: (row) => row.updated_at },
	];
</script>

{#if data.issues.ok}
	{@const issues = data.issues.data.issues}
	<PageHeader title={data.title} intro={m.admin_project_issues_intro()} meta={m.admin_project_issues_meta({ count: String(issues.length) })} {back} />
	{#if issues.length === 0}
		<EmptyState icon={CircleDot} title={m.admin_project_issues_empty_title()} body={m.admin_project_issues_empty_body()} />
	{:else}
		<AdminTable label={m.admin_project_issues_table()} {columns} rows={issues} rowKey={(row) => String(row.number)} muted={(row) => row.status === 'closed'}>
			{#snippet cell(row, column)}
				{#if column.key === 'number'}
					<span class="font-mono text-text-2">#{row.number}</span>
				{:else if column.key === 'title'}
					<span class="text-text">{row.title}</span>
				{:else if column.key === 'kind'}
					<span class="font-mono text-label text-text-2">{row.kind}</span>
				{:else if column.key === 'severity'}
					<span class="font-mono text-label text-text-2">{row.severity}</span>
				{:else if column.key === 'status'}
					<Tag kind={row.status === 'open' ? 'go' : 'neutral'} label={row.status === 'open' ? m.admin_issue_open() : m.admin_issue_closed()} />
				{:else if column.key === 'column'}
					<span class="text-text-2">{row.column_name ?? '—'}</span>
				{:else if column.key === 'updated'}
					<span class="font-mono text-label text-text-2">{dateOnly(row.updated_at, language)}</span>
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<PageHeader title={m.admin_page_projects()} {back} />
	<LoadFailed title={m.admin_project_issues_failed()} problem={data.issues.problem} />
{/if}
