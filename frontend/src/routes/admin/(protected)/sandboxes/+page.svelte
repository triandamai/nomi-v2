<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	// Admin → Sandboxes: every sandbox each project has had, whatever its state (running, waking
	// up, stopped, idle…), and which is the project's live one. `?project=` shows one project's.
	import { getLocale } from '$lib/paraglide/runtime';
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import { phaseLabel } from '$lib/adminProjects';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const when = (iso: string) => new Intl.DateTimeFormat(getLocale(), { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(iso));
	const projectName = $derived(data.project ? (data.sandboxes?.find((s) => s.project_id === data.project)?.project_name ?? data.project) : null);

	const columns = [
		{ key: 'project', label: m.admin_projects_col_project() },
		{ key: 'sandbox', label: m.admin_sandboxes_col_sandbox() },
		{ key: 'state', label: m.admin_sandboxes_col_state() },
		{ key: 'record', label: m.admin_sandboxes_col_record() },
		{ key: 'created', label: m.admin_sandboxes_col_created() },
		{ key: 'updated', label: m.admin_sandboxes_col_updated() },
	];
</script>

<PageHeader title={m.admin_sandboxes()} lede={m.admin_sandboxes_lede()} agent="supervisor" />

{#if projectName}
	<p class="filter md-body-medium">
		<span>{m.admin_sandboxes_showing({ project: projectName })}</span>
		<a class="cell-link" href="/admin/sandboxes">{m.admin_sandboxes_clear()}</a>
	</p>
{/if}

{#if data.sandboxes === null}
	<p class="md-body-medium note" role="alert">{m.admin_sandboxes_failed()}</p>
{:else if data.sandboxes.length === 0}
	<p class="md-body-medium note">{m.admin_sandboxes_empty()}</p>
{:else}
	<DataTable card {columns}>
		{#each data.sandboxes as sandbox (sandbox.id)}
			<tr>
				<td class="md-body-medium">{sandbox.project_name}</td>
				<td class="md-body-medium mono">{sandbox.sandbox_id}</td>
				<td class="md-body-medium"><span class="phase" data-phase={sandbox.phase}>{phaseLabel(sandbox.phase)}</span></td>
				<td class="md-body-medium">{sandbox.current ? m.admin_sandboxes_live() : m.admin_sandboxes_ended()}</td>
				<td class="md-body-small note">{when(sandbox.created_at)}</td>
				<td class="md-body-small note">{when(sandbox.ended_at ?? sandbox.updated_at)}</td>
			</tr>
		{/each}
	</DataTable>
{/if}

<style>
	.filter {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		align-items: center;
		margin-bottom: 16px;
	}
	.note {
		color: var(--md-sys-color-on-surface-variant);
	}
	.mono {
		font-family: var(--md-ref-typeface-mono);
	}
	.phase {
		display: inline-block;
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.phase[data-phase='running'] {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.phase[data-phase='waking'] {
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.phase[data-phase='error'] {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.cell-link {
		color: var(--md-sys-color-primary);
		text-decoration: underline;
		text-underline-offset: 3px;
		min-height: 44px;
		display: inline-flex;
		align-items: center;
	}
	.cell-link:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
</style>
