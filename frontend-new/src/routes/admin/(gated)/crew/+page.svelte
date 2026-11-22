<script lang="ts">
	import { ToggleRight } from '@lucide/svelte';
	import { enhance } from '$app/forms';
	import { m } from '$lib/paraglide/messages';
	import { can } from '$lib/admin/access';
	import type { AdminCrewMember } from '$lib/admin/api';
	import { crewOf } from '$lib/adapters/shell';
	import PageHeader from '$lib/ds/admin/PageHeader.svelte';
	import AdminTable, { type TableColumn } from '$lib/ds/admin/AdminTable.svelte';
	import Mascot from '$lib/ds/agents/Mascot.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';
	import Button from '$lib/ds/actions/Button.svelte';
	import Link from '$lib/ds/actions/Link.svelte';
	import EmptyState from '$lib/ds/lists/EmptyState.svelte';
	import LoadFailed from '$lib/admin/ui/LoadFailed.svelte';
	import Outcome from '$lib/admin/ui/Outcome.svelte';

	// Agent switches (agents: view): every built-in crew member, switched on or off for everyone, as
	// the old app's Admin → Agents. Nomi is always on. With write: turn one off or back on. Members
	// made here are Custom agents.
	let { data, form } = $props();

	const canWrite = $derived(can(data.access, 'agents', 'write'));

	const columns: TableColumn<AdminCrewMember>[] = [
		{ key: 'name', label: m.admin_col_name(), sort: (row) => row.name },
		{ key: 'type', label: m.admin_crew_col_type(), phone: false },
		{ key: 'role', label: m.admin_crew_col_role(), phone: false },
		{ key: 'on', label: m.admin_col_status(), align: 'end' },
	];
</script>

<PageHeader title={m.admin_page_crew()} intro={m.admin_crew_intro()} />

<Outcome {form} action="toggle" failed={m.admin_crew_toggle_failed()} />

{#if data.crew.ok}
	{#if data.crew.data.length === 0}
		<EmptyState icon={ToggleRight} title={m.admin_crew_empty_title()} body={m.admin_crew_empty_body()} />
	{:else}
		<AdminTable label={m.admin_crew_table()} {columns} rows={data.crew.data} rowKey={(row) => row.agent_type} muted={(row) => !row.enabled}>
			{#snippet cell(row, column)}
				{#if column.key === 'name'}
					{@const crew = crewOf(row.agent_type)}
					<span class="flex items-center gap-2">
						{#if crew}<Mascot agent={crew} size={20} live={false} />{/if}
						<span class="font-medium text-text">{row.name}</span>
					</span>
				{:else if column.key === 'type'}
					<span class="font-mono text-label text-text-2">{row.agent_type}</span>
				{:else if column.key === 'role'}
					<span class="text-text-2">{row.role}</span>
				{:else if column.key === 'on'}
					{#if row.locked}
						<Tag kind="neutral" label={m.admin_crew_always_on()} />
					{:else if canWrite}
						<form method="POST" action="?/toggle" use:enhance class="flex justify-end">
							<input type="hidden" name="agent_type" value={row.agent_type} />
							<input type="hidden" name="enabled" value={row.enabled ? 'false' : 'true'} />
							<Button
								label={row.enabled ? m.admin_model_switch_off() : m.admin_model_switch_on()}
								variant="ghost"
								type="submit"
								aria-label={row.enabled ? m.admin_agent_turn_off({ agent: row.name }) : m.admin_agent_turn_on({ agent: row.name })}
							/>
						</form>
					{:else}
						<span class="font-mono text-label text-text-2">{row.enabled ? m.admin_model_on() : m.admin_model_off()}</span>
					{/if}
				{/if}
			{/snippet}
		</AdminTable>
		<p class="text-caption text-text-3">{m.admin_crew_custom()} <Link href="/admin/dynamic-agents" label={m.admin_page_dynamic_agents()} variant="inline" /></p>
	{/if}
{:else}
	<LoadFailed title={m.admin_crew_failed()} problem={data.crew.problem} />
{/if}
