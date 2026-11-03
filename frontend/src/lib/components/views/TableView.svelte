<!-- A table view: comparable figures in the design system's table, anything else in the chat's table (a table where the bubble is wide, a list on phones);
     columns with a format get their values formatted first. -->
<script lang="ts">
	import TableBlock from '$lib/components/blocks/TableBlock.svelte';
	import DataTable from '$lib/components/ds/chat/DataTable.svelte';
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
	// The design system's table (Chat responses → Table) is for comparable rows: a named table whose
	// columns after the first are figures (numbers, money, percentages, dates). Others keep the
	// chat's own table, which has room for text.
	const comparable = $derived(!!view.title && view.columns.length > 1 && view.columns.slice(1).every((c) => c.format && c.format !== 'text'));
	const text = (value: unknown) => (value === null || value === undefined ? '' : String(value));
</script>

{#if comparable && view.title}
	<DataTable title={view.title} columns={view.columns.map((c) => c.label)} rows={rows.map((row) => view.columns.map((c) => text(row[c.key])))} />
{:else}
{#if view.title}
	<ViewFrame title={view.title}>
		<TableBlock block={{ kind: 'table', variant: 'data', columns: view.columns, rows }} {moneyKeys} />
	</ViewFrame>
{:else}
	<TableBlock block={{ kind: 'table', variant: 'data', columns: view.columns, rows }} {moneyKeys} />
{/if}
{/if}
