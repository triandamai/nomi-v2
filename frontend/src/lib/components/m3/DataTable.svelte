<script lang="ts">
	import type { Snippet } from 'svelte';
	import Pagination from './Pagination.svelte';
	import IconChevronDown from '../icons/IconChevronDown.svelte';
	import IconChevronUp from '../icons/IconChevronUp.svelte';
	import IconSearch from '../icons/IconSearch.svelte';

	let {
		columns,
		sortKey = $bindable<string | undefined>(undefined),
		sortDirection = $bindable<'asc' | 'desc'>('asc'),
		onSort,
		page = $bindable(1),
		pageSize = 20,
		totalItems,
		onPageChange,
		searchQuery = $bindable(''),
		onSearch,
		searchPlaceholder = 'Search...',
		children,
		card = false,
		class: extraClass = '',
	}: {
		columns: { key: string; label: string; sortable?: boolean }[];
		sortKey?: string;
		sortDirection?: 'asc' | 'desc';
		onSort?: (key: string) => void;
		page?: number;
		pageSize?: number;
		totalItems?: number;
		onPageChange?: (page: number) => void;
		searchQuery?: string;
		onSearch?: (query: string) => void;
		searchPlaceholder?: string;
		children: Snippet;
		/** Sets the table on its own rounded surface (a page's main table, not one inside a bubble). */
		card?: boolean;
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

	const totalPages = $derived(totalItems !== undefined ? Math.max(1, Math.ceil(totalItems / pageSize)) : undefined);

	function goToPage(next: number) {
		if (totalPages === undefined) return;
		const clamped = Math.min(Math.max(1, next), totalPages);
		if (clamped === page) return;
		page = clamped;
		onPageChange?.(page);
	}

	const SEARCH_DEBOUNCE_MS = 300;
	let searchDebounceTimer: ReturnType<typeof setTimeout> | undefined;

	// On a phone each row becomes a card: every cell is labelled with its column's name, read
	// from the header so pages don't repeat it.
	function labelCells(tbody: HTMLTableSectionElement) {
		const label = () => {
			for (const row of tbody.rows) {
				[...row.cells].forEach((cell, i) => {
					const name = columns[i]?.label ?? '';
					if (cell.dataset.label !== name) cell.dataset.label = name;
				});
			}
		};
		label();
		const observer = new MutationObserver(label);
		observer.observe(tbody, { childList: true, subtree: true });
		return { destroy: () => observer.disconnect() };
	}

	function handleSearchInput(value: string) {
		searchQuery = value;
		clearTimeout(searchDebounceTimer);
		searchDebounceTimer = setTimeout(() => onSearch?.(searchQuery), SEARCH_DEBOUNCE_MS);
	}

	function handleSearchKeydown(event: KeyboardEvent) {
		if (event.key !== 'Enter') return;
		clearTimeout(searchDebounceTimer);
		onSearch?.(searchQuery);
	}
</script>

<div class="m3-data-table-wrap {extraClass}">
	{#if onSearch}
		<div class="m3-data-table__search">
			<IconSearch size={18} />
			<input
				type="text"
				value={searchQuery}
				placeholder={searchPlaceholder}
				oninput={(event) => handleSearchInput(event.currentTarget.value)}
				onkeydown={handleSearchKeydown}
				aria-label={searchPlaceholder}
			/>
		</div>
	{/if}
	<table class="m3-data-table" class:m3-data-table--card={card}>
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
									{#if sortDirection === 'asc'}
										<IconChevronUp size={14} />
									{:else}
										<IconChevronDown size={14} />
									{/if}
								{/if}
							</button>
						{:else}
							{column.label}
						{/if}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody use:labelCells>
			{@render children()}
		</tbody>
	</table>
	{#if totalPages !== undefined && totalPages > 1}
		<div class="m3-data-table__pagination">
			<span class="m3-data-table__pagination-label">Page {page} of {totalPages}</span>
			<Pagination {page} pages={totalPages} onselect={goToPage} />
		</div>
	{/if}
</div>

<style>
	.m3-data-table-wrap {
		overflow-x: auto;
	}
	.m3-data-table {
		width: 100%;
		border-collapse: separate;
		border-spacing: 0;
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}
	.m3-data-table--card {
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		overflow: hidden;
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
	.m3-data-table :global(tbody tr:last-child td) {
		border-bottom: none;
	}
	.m3-data-table :global(tbody tr:hover) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent);
	}

	.m3-data-table__search {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		max-width: 320px;
		box-sizing: border-box;
		margin-bottom: 12px;
		padding: 0 12px;
		height: 40px;
		border-radius: var(--md-sys-shape-corner-full);
		border: 1px solid var(--md-sys-color-outline);
		color: var(--md-sys-color-on-surface-variant);
	}
	.m3-data-table__search:focus-within {
		border: 2px solid var(--md-sys-color-primary);
		padding: 0 11px;
	}
	.m3-data-table__search input {
		flex: 1;
		border: none;
		background: transparent;
		outline: none;
		color: var(--md-sys-color-on-surface);
		font-family: var(--md-sys-typescale-body-medium-font);
		font-size: var(--md-sys-typescale-body-medium-size);
	}

	.m3-data-table__pagination {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 12px;
		margin-top: 8px;
	}
	.m3-data-table__pagination-label {
		font-family: var(--md-sys-typescale-body-small-font);
		font-size: var(--md-sys-typescale-body-small-size);
		color: var(--md-sys-color-on-surface-variant);
	}
	/* Phone: rows become cards, each cell a "Column: value" line. */
	@media (max-width: 640px) {
		.m3-data-table--card {
			border-radius: 0;
			background: transparent;
		}
		.m3-data-table thead {
			display: none;
		}
		.m3-data-table :global(tbody) {
			display: flex;
			flex-direction: column;
			gap: 8px;
		}
		.m3-data-table :global(tbody tr) {
			display: flex;
			flex-direction: column;
			gap: 2px;
			padding: 14px 16px;
			border-radius: var(--md-sys-shape-corner-large);
			background: var(--md-sys-color-surface-container-lowest);
		}
		.m3-data-table :global(tbody td) {
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 12px;
			min-height: 32px;
			padding: 0;
			border-bottom: none;
			overflow-wrap: anywhere;
		}
		/* A cell with no column name (row actions) sits at the card's end. */
		.m3-data-table :global(tbody td[data-label='']) {
			justify-content: flex-end;
		}
		.m3-data-table :global(tbody td[data-label]:not([data-label='']))::before {
			content: attr(data-label);
			flex: none;
			color: var(--md-sys-color-on-surface-variant);
			font-family: var(--md-ref-typeface-mono);
			font-size: 0.6875rem;
			letter-spacing: 0.06em;
			text-transform: uppercase;
		}
		.m3-data-table__pagination {
			justify-content: space-between;
		}
	}
</style>
