<script lang="ts">
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	type Row = {
		agent_session_id: string;
		user_label: string;
		agent_type: string;
		channel: string;
		started_at: string;
		last_activity_at: string;
	};

	const rows: Row[] = $derived(
		data.agents.users.flatMap((group) =>
			group.agents.map((agent) => ({
				agent_session_id: agent.agent_session_id,
				user_label: group.label,
				agent_type: agent.agent_type,
				channel: agent.channel,
				started_at: agent.started_at,
				last_activity_at: agent.last_activity_at,
			})),
		),
	);

	const columns = [
		{ key: 'user_label', label: 'User', sortable: true },
		{ key: 'agent_type', label: 'Agent Type', sortable: true },
		{ key: 'channel', label: 'Channel', sortable: true },
		{ key: 'started_at', label: 'Started', sortable: true },
		{ key: 'last_activity_at', label: 'Last Activity', sortable: true },
	];

	let sortKey = $state<string | undefined>('user_label');
	let sortDirection = $state<'asc' | 'desc'>('asc');

	const sortedRows = $derived.by(() => {
		if (!sortKey) return rows;
		const key = sortKey as keyof Row;
		const direction = sortDirection === 'asc' ? 1 : -1;
		return [...rows].sort((a, b) => {
			const av = a[key];
			const bv = b[key];
			if (av < bv) return -1 * direction;
			if (av > bv) return 1 * direction;
			return 0;
		});
	});
</script>

<h1 class="md-headline-small-emphasized" style="color: var(--md-sys-color-on-surface)">Running agents</h1>
<p class="md-body-large mt-2" style="color: var(--md-sys-color-on-surface-variant)">
	Every currently active agent session.
</p>

<div class="mt-6">
	{#if sortedRows.length === 0}
		<p class="md-body-medium" style="color: var(--md-sys-color-on-surface-variant)">
			No agents are currently running.
		</p>
	{:else}
		<DataTable {columns} bind:sortKey bind:sortDirection>
			{#each sortedRows as row (row.agent_session_id)}
				<tr>
					<td>{row.user_label}</td>
					<td>{row.agent_type}</td>
					<td>{row.channel}</td>
					<td>{new Date(row.started_at).toLocaleString()}</td>
					<td>{new Date(row.last_activity_at).toLocaleString()}</td>
				</tr>
			{/each}
		</DataTable>
	{/if}
</div>
