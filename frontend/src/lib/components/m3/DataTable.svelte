<script lang="ts">
	import type { Snippet } from 'svelte';
	import Icon from './Icon.svelte';

	let {
		columns,
		sortKey = $bindable<string | undefined>(undefined),
		sortDirection = $bindable<'asc' | 'desc'>('asc'),
		onSort,
		children,
		class: extraClass = '',
	}: {
		columns: { key: string; label: string; sortable?: boolean }[];
		sortKey?: string;
		sortDirection?: 'asc' | 'desc';
		onSort?: (key: string) => void;
		children: Snippet;
		class?: string;
	} = $props();

	function handleSort(column: { key: string; sortable?: boolean }) {
		if (!column.sortable) return;
		if (sortKey === column.key) {
			sortDirection = sortDirection === 'asc' ? 'desc' : 'asc';
		} else {
			sortKey = column.key;
			sortDirection = 'asc';
		}
		onSort?.(column.key);
	}
</script>

<div class="m3-data-table-wrap {extraClass}">
	<table class="m3-data-table">
		<thead>
			<tr>
				{#each columns as column (column.key)}
					<th
						scope="col"
						aria-sort={sortKey === column.key ? (sortDirection === 'asc' ? 'ascending' : 'descending') : 'none'}
					>
						{#if column.sortable}
							<button type="button" class="m3-data-table__sort" onclick={() => handleSort(column)}>
								{column.label}
								{#if sortKey === column.key}
									<Icon name={sortDirection === 'asc' ? 'chevron-up' : 'chevron-down'} size={14} />
								{/if}
							</button>
						{:else}
							{column.label}
						{/if}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{@render children()}
		</tbody>
	</table>
</div>

<style>
	.m3-data-table-wrap {
		overflow-x: auto;
	}
	.m3-data-table {
		width: 100%;
		border-collapse: collapse;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.m3-data-table thead th {
		text-align: left;
		padding: 12px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-sys-typescale-label-large-font);
		font-weight: var(--md-sys-typescale-label-large-weight);
		font-size: var(--md-sys-typescale-label-large-size);
		white-space: nowrap;
	}
	.m3-data-table__sort {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		border: none;
		background: transparent;
		cursor: pointer;
		padding: 0;
		color: inherit;
		font: inherit;
	}
	/* tbody rows/cells come from the consumer's `children` snippet, not this component's own
	   template — Svelte's scoped-style hash only applies to markup this file renders directly,
	   so styling externally-provided content needs :global(). */
	.m3-data-table :global(tbody td) {
		padding: 12px 16px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		color: var(--md-sys-color-on-surface);
	}
	.m3-data-table :global(tbody tr:hover) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
	}
</style>
