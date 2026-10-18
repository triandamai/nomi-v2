<!-- A table view, drawn by the chat's table (a table where the bubble is wide, a list on phones);
     columns with a format get their values formatted first. -->
<script lang="ts">
	import TableBlock from '$lib/components/blocks/TableBlock.svelte';
	import ViewFrame from './ViewFrame.svelte';
	import { getLocale } from '$lib/paraglide/runtime';
	import { formatValue } from '$lib/views/format';
	import type { Scalar, TableView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<TableView> = $props();

	const rows = $derived(
		view.rows.map((row) =>
			Object.fromEntries(
				view.columns.map((column) => {
					const value = row[column.key];
					const shown = value === null || value === undefined || typeof value === 'object' ? (value ?? '') : formatValue(value as Scalar, column.format, getLocale());
					return [column.key, shown];
				}),
			),
		),
	);
	const moneyKeys = $derived(view.columns.filter((c) => c.format === 'money').map((c) => c.key));
</script>

{#if view.title}
	<ViewFrame title={view.title}>
		<TableBlock block={{ kind: 'table', variant: 'data', columns: view.columns, rows }} {moneyKeys} />
	</ViewFrame>
{:else}
	<TableBlock block={{ kind: 'table', variant: 'data', columns: view.columns, rows }} {moneyKeys} />
{/if}
