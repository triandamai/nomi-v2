<script lang="ts">
	import { Box } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { AdminSandbox } from '$lib/admin/api';
	import { dateTime } from '$lib/admin/format';
	import { phaseLabel } from '$lib/admin/projects';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Link from '$lib/ds/actions/Link.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';

	// Sandboxes (projects: view): every sandbox each project has had, whatever its state (running,
	// waking up, stopped, idle…), and which is the project's live one. `?project=` shows one
	// project's. An ended record keeps the last state it had, shown muted so it never reads as live.
	let { data } = $props();

	const language = getLocale();
	const projectName = $derived(data.project && data.sandboxes.ok ? (data.sandboxes.data.find((one) => one.project_id === data.project)?.project_name ?? data.project) : null);

	const columns: TableColumn<AdminSandbox>[] = [
		{ key: 'project', label: m.admin_col_project(), sort: (row) => row.project_name },
		{ key: 'sandbox', label: m.admin_sandboxes_col_sandbox(), phone: false },
		{ key: 'state', label: m.admin_sandboxes_col_state() },
		{ key: 'record', label: m.admin_sandboxes_col_record(), phone: false },
		{ key: 'created', label: m.admin_sandboxes_col_created(), phone: false, sort: (row) => row.created_at },
		{ key: 'updated', label: m.admin_sandboxes_col_updated(), sort: (row) => row.ended_at ?? row.updated_at },
	];
</script>

<PageHeader title={m.admin_page_sandboxes()} intro={m.admin_sandboxes_intro()} />

{#if projectName}
	<div class="flex flex-wrap items-center gap-3">
		<span class="text-ui text-text-2">{m.admin_sandboxes_showing({ project: projectName })}</span>
		<Link href="/admin/sandboxes" label={m.admin_sandboxes_clear()} />
	</div>
{/if}

{#if data.sandboxes.ok}
	{#if data.sandboxes.data.length === 0}
		<EmptyState icon={Box} title={m.admin_sandboxes_empty_title()} body={m.admin_sandboxes_empty_body()} />
	{:else}
		<AdminTable label={m.admin_sandboxes_table()} {columns} rows={data.sandboxes.data} rowKey={(row) => row.id} muted={(row) => !row.current}>
			{#snippet cell(row, column)}
				{#if column.key === 'project'}
					<span class="font-mono font-medium text-text">{row.project_name}</span>
				{:else if column.key === 'sandbox'}
					<span class="font-mono text-label text-text-2">{row.sandbox_id}</span>
				{:else if column.key === 'state'}
					<Tag kind={!row.current ? 'neutral' : row.phase === 'error' ? 'no-go' : row.phase === 'running' ? 'go' : 'neutral'} label={phaseLabel(row.phase)} />
				{:else if column.key === 'record'}
					<span class="text-text-2">{row.current ? m.admin_sandboxes_live() : m.admin_sandboxes_ended()}</span>
				{:else if column.key === 'created'}
					<span class="font-mono text-label text-text-2">{dateTime(row.created_at, language)}</span>
				{:else if column.key === 'updated'}
					<span class="font-mono text-label text-text-2">{dateTime(row.ended_at ?? row.updated_at, language)}</span>
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_sandboxes_failed()} problem={data.sandboxes.problem} />
{/if}
