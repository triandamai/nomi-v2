<script lang="ts">
	import { Bot, Plus } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { DynamicAgent } from '$lib/admin/api';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Custom agents (agents: view; write changes): the crew members made here, what each is for,
	// the tools it may use, and whether it's on. One opens to change it.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'agents', 'write'));
	const columns: TableColumn<DynamicAgent>[] = [
		{ key: 'name', label: m.admin_col_name(), sort: (row) => row.name },
		{ key: 'for', label: m.admin_col_for(), phone: false },
		{ key: 'tools', label: m.admin_col_tools(), phone: false },
		{ key: 'on', label: m.admin_col_on(), align: 'end' },
	];
</script>

<PageHeader title={m.admin_page_dynamic_agents()} intro={m.admin_agents_intro()}>
	{#snippet actions()}
		{#if canWrite}<Button label={m.admin_agents_add()} variant="inverse" iconStart={Plus} href="/admin/dynamic-agents/new" />{/if}
	{/snippet}
</PageHeader>

<Outcome {form} action="toggle" failed={m.admin_agent_toggle_failed()} />

{#if data.agents.ok}
	{#if data.agents.data.length === 0}
		<EmptyState icon={Bot} title={m.admin_agents_empty_title()} body={m.admin_agents_empty_body()} />
	{:else}
		<AdminTable label={m.admin_agents_table()} {columns} rows={data.agents.data} rowKey={(row) => row.id} muted={(row) => !row.is_active}>
			{#snippet cell(row, column)}
				{#if column.key === 'name'}
					<a href="/admin/dynamic-agents/{row.id}" class="rounded-xs font-medium text-text outline-offset-2 hover:underline">{row.name}</a>
				{:else if column.key === 'for'}
					{row.intent_label}
				{:else if column.key === 'tools'}
					<span class="font-mono text-label text-text-2">{row.granted_tools.join(', ') || '—'}</span>
				{:else if column.key === 'on'}
					{#if canWrite}
						<form method="POST" action="?/toggle" use:enhance class="flex justify-end">
							<input type="hidden" name="id" value={row.id} />
							<Button
								label={row.is_active ? m.admin_model_switch_off() : m.admin_model_switch_on()}
								variant="ghost"
								type="submit"
								aria-label={row.is_active ? m.admin_agent_turn_off({ agent: row.name }) : m.admin_agent_turn_on({ agent: row.name })}
							/>
						</form>
					{:else}
						<span class="font-mono text-label text-text-2">{row.is_active ? m.admin_model_on() : m.admin_model_off()}</span>
					{/if}
				{/if}
			{/snippet}
		</AdminTable>
	{/if}
{:else}
	<LoadFailed title={m.admin_agents_failed()} problem={data.agents.problem} />
{/if}
