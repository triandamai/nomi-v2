<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Admin → Projects: every project with its owner, its sandbox (how many it has had, and
	// whether one is active) and its issues; each count opens the page about it.
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { issueSummary, sandboxSummary } from '$lib/adminProjects';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const columns = [
		{ key: 'project', label: m.admin_projects_col_project() },
		{ key: 'owner', label: m.admin_projects_col_owner() },
		{ key: 'sandbox', label: m.admin_projects_col_sandbox() },
		{ key: 'issues', label: m.admin_projects_col_issues() },
	];
</script>

<PageHeader title={m.admin_projects()} lede={m.admin_projects_lede()} agent="supervisor" />

<form method="GET" class="search" role="search">
	<input class="search__field md-body-medium" type="search" name="q" value={data.q} placeholder={m.admin_projects_search()} aria-label={m.admin_projects_search()} />
</form>

{#if data.projects === null}
	<p class="md-body-medium note" role="alert">{m.admin_projects_failed()}</p>
{:else if data.projects.length === 0}
	<p class="md-body-medium note">{m.admin_projects_empty()}</p>
{:else}
	<DataTable card {columns}>
		{#each data.projects as project (project.id)}
			<tr>
				<td>
					<span class="md-title-small">{project.name}</span>
					{#if project.workspace}<span class="md-body-small note block">{project.workspace}</span>{/if}
				</td>
				<td class="md-body-medium">{project.owner_email ?? '—'}</td>
				<td><a class="cell-link md-body-medium" href={`/admin/sandboxes?project=${project.id}`} aria-label={m.admin_projects_sandbox_link({ project: project.name, summary: sandboxSummary(project) })}>{sandboxSummary(project)}</a></td>
				<td><a class="cell-link md-body-medium" href={`/admin/projects/${project.id}/issues`} aria-label={m.admin_projects_issues_link({ project: project.name, summary: issueSummary(project) })}>{issueSummary(project)}</a></td>
			</tr>
		{/each}
	</DataTable>
{/if}

<style>
	.search {
		margin-bottom: 16px;
	}
	.search__field {
		width: min(100%, 360px);
		min-height: 44px;
		padding: 0 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.search__field:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.note {
		color: var(--md-sys-color-on-surface-variant);
	}
	.block {
		display: block;
	}
	.cell-link {
		color: var(--md-sys-color-primary);
		text-decoration: underline;
		text-underline-offset: 3px;
	}
	.cell-link:hover {
		text-decoration-thickness: 2px;
	}
	.cell-link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
		border-radius: 2px;
	}
</style>
