<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import type { Snippet } from 'svelte';
	import IconChevronDown from '../icons/IconChevronDown.svelte';
	import IconChevronLeft from '../icons/IconChevronLeft.svelte';
	import IconChevronRight from '../icons/IconChevronRight.svelte';
	import IconFirstPage from '../icons/IconFirstPage.svelte';
	import IconLastPage from '../icons/IconLastPage.svelte';
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
		pageSizeOptions,
		onPageSizeChange,
		searchQuery = $bindable(''),
		onSearch,
		searchPlaceholder = m.common_search(),
		children,
		list,
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
		/** Offer a "Rows per page" choice (with onPageSizeChange). */
		pageSizeOptions?: number[];
		onPageSizeChange?: (size: number) => void;
		searchQuery?: string;
		onSearch?: (query: string) => void;
		searchPlaceholder?: string;
		children: Snippet;
		/** What phones show instead of the table: the rows as list items (see ListItem). */
		list?: Snippet;
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

	const firstRow = $derived(totalItems ? (page - 1) * pageSize + 1 : 0);
	const lastRow = $derived(totalItems ? Math.min(page * pageSize, totalItems) : 0);
	const showFooter = $derived(totalItems !== undefined && (totalPages! > 1 || (pageSizeOptions !== undefined && totalItems > Math.min(...pageSizeOptions))));

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
	<table class="m3-data-table" class:m3-data-table--card={card} class:m3-data-table--has-list={!!list}>
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
	{#if list}
		<div role="list" class="m3-data-table__list">
			{@render list()}
		</div>
	{/if}
	{#if showFooter && totalPages !== undefined}
		<div class="m3-data-table__footer">
			{#if pageSizeOptions && onPageSizeChange}
				<label class="m3-data-table__size">
					<span>{m.page_rows()}</span>
					<span class="m3-data-table__select">
						<select value={pageSize} onchange={(event) => onPageSizeChange(Number(event.currentTarget.value))}>
							{#each pageSizeOptions as option (option)}
								<option value={option}>{option}</option>
							{/each}
						</select>
						<IconChevronDown size={16} />
					</span>
				</label>
			{/if}
			<span class="m3-data-table__range" aria-live="polite">{m.page_range({ first: firstRow, last: lastRow, total: totalItems ?? 0 })}</span>
			<nav class="m3-data-table__pager" aria-label={m.page_pages()}>
				<button type="button" class="m3-data-table__nav" onclick={() => goToPage(1)} disabled={page <= 1} aria-label={m.page_first()}><IconFirstPage /></button>
				<button type="button" class="m3-data-table__nav" onclick={() => goToPage(page - 1)} disabled={page <= 1} aria-label={m.page_prev()}><IconChevronLeft /></button>
				<button type="button" class="m3-data-table__nav" onclick={() => goToPage(page + 1)} disabled={page >= totalPages} aria-label={m.page_next()}><IconChevronRight /></button>
				<button type="button" class="m3-data-table__nav" onclick={() => goToPage(totalPages)} disabled={page >= totalPages} aria-label={m.page_last()}><IconLastPage /></button>
			</nav>
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

	/* M3 data table footer: rows per page, the range shown, and page arrows. */
	.m3-data-table__footer {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		flex-wrap: wrap;
		gap: 8px 24px;
		margin-top: 4px;
		padding: 8px 4px 0 16px;
		color: var(--md-sys-color-on-surface-variant);
		font-family: var(--md-sys-typescale-body-small-font);
		font-size: var(--md-sys-typescale-body-small-size);
	}
	.m3-data-table--card ~ .m3-data-table__footer {
		margin-top: 8px;
		padding: 6px 8px 6px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.m3-data-table__size {
		display: inline-flex;
		align-items: center;
		gap: 8px;
	}
	.m3-data-table__select {
		position: relative;
		display: inline-flex;
		align-items: center;
	}
	.m3-data-table__select select {
		appearance: none;
		height: 32px;
		padding: 0 28px 0 12px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-variant-numeric: tabular-nums;
		cursor: pointer;
	}
	.m3-data-table__select select:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.m3-data-table__select :global(svg) {
		position: absolute;
		right: 8px;
		pointer-events: none;
	}
	.m3-data-table__range {
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.m3-data-table__pager {
		display: inline-flex;
		gap: 2px;
	}
	.m3-data-table__nav {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 40px;
		height: 40px;
		border: none;
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface-variant);
		cursor: pointer;
		transition:
			background-color var(--nomi-motion-effects-fast, 150ms),
			border-radius var(--nomi-motion-spatial-fast, 200ms);
	}
	.m3-data-table__nav:hover:not(:disabled) {
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent);
	}
	.m3-data-table__nav:active:not(:disabled) {
		border-radius: var(--md-sys-shape-corner-medium);
		background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent);
	}
	.m3-data-table__nav:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.m3-data-table__nav:disabled {
		opacity: 0.38;
		cursor: default;
	}
	/* The phone list is hidden until a phone shows it in place of the table. */
	.m3-data-table__list {
		display: none;
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
		/* With a list to show, the table steps aside. */
		.m3-data-table--has-list {
			display: none;
		}
		.m3-data-table__list {
			display: flex;
			flex-direction: column;
			gap: 2px;
			padding: 4px;
			border-radius: var(--md-sys-shape-corner-extra-large);
			background: var(--md-sys-color-surface-container-lowest);
		}
		.m3-data-table__footer,
		.m3-data-table--card ~ .m3-data-table__footer {
			justify-content: space-between;
			padding: 6px 4px 6px 16px;
		}
		.m3-data-table__pager {
			margin-left: auto;
		}
	}
</style>
