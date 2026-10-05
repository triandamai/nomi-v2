<!-- frontend/src/lib/components/blocks/TableBlock.svelte -->
<script lang="ts">
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import { categoryLook } from '$lib/money';
	import { tableListColumns, tableListItem } from '$lib/tableList';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'table' }> } = $props();

	const listColumns = $derived(tableListColumns(block.columns, block.rows));
</script>

{#if block.variant === 'data'}
	<!-- A table where the bubble is wide; a list where it is narrow (phones), so nothing scrolls sideways. -->
	<div class="table-block">
		<div class="table-block__table">
			<DataTable columns={block.columns}>
				{#each block.rows as row, i (i)}
					<tr>
						{#each block.columns as column (column.key)}
							<td>{row[column.key] ?? ''}</td>
						{/each}
					</tr>
				{/each}
			</DataTable>
		</div>
		<ul class="table-block__list">
			{#each block.rows as row, i (i)}
				{@const item = tableListItem(row, listColumns)}
				<li class="table-row">
					{#if item.category}
						{@const look = categoryLook(item.category)}
						<AgentShape shape={look.shape} tone={look.tone} size={36} />
					{/if}
					<div class="table-row__text">
						<span class="table-row__headline">{item.headline}</span>
						{#if item.category || item.supporting.length > 0}
							<span class="table-row__supporting">{[item.category, ...item.supporting].filter(Boolean).join(' · ')}</span>
						{/if}
					</div>
					{#if item.trailing}
						<span class="table-row__trailing">{item.trailing}</span>
					{/if}
				</li>
			{/each}
		</ul>
	</div>
{:else}
	<div class="m3-comparison-grid">
		{#each block.rows as row, i (i)}
			<Card variant="outlined" class="p-3">
				{#each block.columns as column (column.key)}
					<div class="m3-comparison-field">
						<span class="md-label-small" style="color: var(--md-sys-color-on-surface-variant)">{column.label}</span>
						<span class="md-body-medium">{row[column.key] ?? ''}</span>
					</div>
				{/each}
			</Card>
		{/each}
	</div>
{/if}

<style>
	.table-block {
		container-type: inline-size;
		width: 100%;
		min-width: 0;
	}
	/* Doubled class: a bubble's own list styling (indent) must not reach this list. */
	.table-block .table-block__list {
		display: none;
		margin: 0;
		padding: 4px 0;
		list-style: none;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
	}
	@container (max-width: 480px) {
		.table-block__table {
			display: none;
		}
		.table-block .table-block__list {
			display: block;
		}
	}
	.table-row {
		display: flex;
		align-items: center;
		gap: 12px;
		min-height: 56px;
		margin: 0;
		padding: 8px 12px;
	}
	.table-row + .table-row {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.table-row__text {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
	}
	.table-row__headline {
		display: -webkit-box;
		overflow: hidden;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		color: var(--md-sys-color-on-surface);
		font-size: 1rem;
		line-height: 1.375rem;
		overflow-wrap: anywhere;
	}
	.table-row__supporting {
		display: -webkit-box;
		overflow: hidden;
		-webkit-box-orient: vertical;
		-webkit-line-clamp: 2;
		line-clamp: 2;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
		line-height: 1.25rem;
	}
	.table-row__trailing {
		flex-shrink: 0;
		color: var(--md-sys-color-on-surface);
		font-size: 0.875rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.m3-comparison-grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 8px;
	}
	.m3-comparison-field {
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 4px 0;
	}
</style>
