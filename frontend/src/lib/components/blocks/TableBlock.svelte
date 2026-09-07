<!-- frontend/src/lib/components/blocks/TableBlock.svelte -->
<script lang="ts">
	import DataTable from '$lib/components/m3/DataTable.svelte';
	import Card from '$lib/components/m3/Card.svelte';
	import type { ContentBlock } from '$lib/types';

	let { block }: { block: Extract<ContentBlock, { kind: 'table' }> } = $props();
</script>

{#if block.variant === 'data'}
	<DataTable columns={block.columns}>
		{#each block.rows as row, i (i)}
			<tr>
				{#each block.columns as column (column.key)}
					<td>{row[column.key] ?? ''}</td>
				{/each}
			</tr>
		{/each}
	</DataTable>
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
