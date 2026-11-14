<script lang="ts">
	import AdminTable, { type TableColumn, type TableSort } from './AdminTable.svelte';

	// For tests and the docs: a table of plans whose cells are plain text.
	type Plan = { slug: string; name: string; tokens: number; hidden?: boolean };
	let { rows, sort = $bindable() }: { rows: Plan[]; sort?: TableSort } = $props();

	const columns: TableColumn<Plan>[] = [
		{ key: 'name', label: 'Plan', sort: (row) => row.name },
		{ key: 'slug', label: 'Id', phone: false },
		{ key: 'tokens', label: 'Tokens a month', align: 'end', sort: (row) => row.tokens },
	];
</script>

<AdminTable label="Plans" {columns} {rows} rowKey={(row) => row.slug} bind:sort muted={(row) => Boolean(row.hidden)}>
	{#snippet cell(row, column)}
		{#if column.key === 'name'}{row.name}{:else if column.key === 'slug'}<span class="font-mono text-label text-text-2">{row.slug}</span>{:else}<span class="font-mono text-caption">{row.tokens}</span>{/if}
	{/snippet}
</AdminTable>
