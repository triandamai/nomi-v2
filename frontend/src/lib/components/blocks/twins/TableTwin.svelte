<script lang="ts">
	// A data table the crew posted (DataTable): named, with where its rows came from underneath, and
	// copy and download. A table from before tables had a title keeps the old block: there's
	// nothing true to name it with.
	import DataTable from '$lib/components/ds/chat/DataTable.svelte';
	import TableBlock from '../TableBlock.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block, moneyKeys = [] }: { block: Extract<ContentBlock, { kind: 'table' }>; moneyKeys?: string[] } = $props();

	const text = (value: unknown) => (value === null || value === undefined ? '' : typeof value === 'object' ? JSON.stringify(value) : String(value));
</script>

{#if block.title && block.variant === 'data'}
	<DataTable
		title={block.title}
		columns={block.columns.map((column) => column.label)}
		rows={block.rows.map((row) => block.columns.map((column) => text(row[column.key])))}
		source={block.source}
	/>
{:else}
	<TableBlock {block} {moneyKeys} />
{/if}
