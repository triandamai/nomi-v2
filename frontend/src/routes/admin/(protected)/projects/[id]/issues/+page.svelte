<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Admin → a project's issues, read only: what's open and what was closed, for an admin who
	// isn't one of the project's people. The people work on them in the project itself.
	import { getLocale } from '$lib/paraglide/runtime';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const when = (iso: string) => new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium' }).format(new Date(iso));

	const columns = [
		{ key: 'number', label: '#' },
		{ key: 'title', label: m.admin_project_issues_col_title() },
		{ key: 'kind', label: m.admin_project_issues_col_kind() },
		{ key: 'severity', label: m.admin_project_issues_col_severity() },
		{ key: 'status', label: m.admin_project_issues_col_status() },
		{ key: 'column', label: m.admin_project_issues_col_column() },
		{ key: 'updated', label: m.admin_project_issues_col_updated() },
	];
</script>

<a class="back md-label-large" href="/admin/projects">← {m.admin_projects()}</a>
<PageHeader title={m.admin_project_issues_title({ project: data.project.name })} lede={m.admin_project_issues_lede()} agent="supervisor" />

{#if data.issues.length === 0}
	<p class="md-body-medium note">{m.admin_project_issues_empty()}</p>
{:else}
	<DataTable card {columns}>
		{#each data.issues as issue (issue.number)}
			<tr>
				<td class="md-body-medium mono">#{issue.number}</td>
				<td class="md-body-medium">{issue.title}</td>
				<td class="md-body-medium">{issue.kind}</td>
				<td class="md-body-medium">{issue.severity}</td>
				<td class="md-body-medium">{issue.status === 'open' ? m.admin_project_issues_open() : m.admin_project_issues_closed()}</td>
				<td class="md-body-medium">{issue.column_name ?? '—'}</td>
				<td class="md-body-small note">{when(issue.updated_at)}</td>
			</tr>
		{/each}
	</DataTable>
{/if}

<style>
	.back {
		display: inline-flex;
		align-items: center;
		min-height: 44px;
		color: var(--md-sys-color-primary);
		text-decoration: none;
	}
	.back:hover {
		text-decoration: underline;
	}
	.back:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.note {
		color: var(--md-sys-color-on-surface-variant);
	}
	.mono {
		font-family: var(--md-ref-typeface-mono);
	}
</style>
