<!-- Plans: names first, numbers in mono at the end, sortable by tokens. -->
<script lang="ts">
	import AdminTable, { type TableColumn, type TableSort } from '$lib/ds/admin/AdminTable.svelte';
	import Tag from '$lib/ds/status/Tag.svelte';

	type Plan = { slug: string; name: string; tokens: string; sort: number; people: number; isDefault?: boolean };
	const rows: Plan[] = [
		{ slug: 'free', name: 'Free', tokens: '1M', sort: 1_000_000, people: 128, isDefault: true },
		{ slug: 'pro', name: 'Pro', tokens: '5M', sort: 5_000_000, people: 21 },
	];
	const columns: TableColumn<Plan>[] = [
		{ key: 'name', label: 'Plan', sort: (row) => row.name },
		{ key: 'tokens', label: 'Tokens a month', align: 'end', sort: (row) => row.sort },
		{ key: 'people', label: 'People', align: 'end', phone: false },
	];
	let sort = $state<TableSort>({ key: 'tokens', direction: 'descending' });
</script>

<div class="w-full max-w-160">
	<AdminTable label="Plans" {columns} {rows} rowKey={(row) => row.slug} bind:sort>
		{#snippet cell(row, column)}
			{#if column.key === 'name'}
				<span class="inline-flex items-center gap-2">{row.name}{#if row.isDefault}<Tag kind="neutral" label="Default" />{/if}</span>
			{:else if column.key === 'tokens'}
				<span class="font-mono text-caption">{row.tokens}</span>
			{:else}
				<span class="font-mono text-caption text-text-2">{row.people}</span>
			{/if}
		{/snippet}
	</AdminTable>
</div>
