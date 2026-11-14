<script lang="ts" module>
	export type TableColumn<Row> = {
		key: string;
		label: string;
		/** Numbers and amounts sit at the end, so their digits line up. */
		align?: 'start' | 'end';
		/** Makes the column sortable by this value. */
		sort?: (row: Row) => string | number;
		/** Left out on a phone, where only what names the row and its main fact fit. */
		phone?: boolean;
	};
	export type TableSort = { key: string; direction: 'ascending' | 'descending' };
</script>

<script lang="ts" generics="Row">
	import type { Snippet } from 'svelte';
	import { ArrowDown, ArrowUp, ArrowUpDown } from '@lucide/svelte';
	import Icon from '../foundations/Icon.svelte';

	// Admin → Data table: rows of the same kind to compare and act on (people, plans, keys), drawn
	// like Chat responses → Table: a flat bordered box, column names as mono section labels, a 1px
	// line between rows, numbers in mono at the end. A column with `sort` sorts on a click of its
	// name (aria-sort says which way); the page renders each cell (`cell`), so links and actions sit
	// in the row. Wider than a phone, it scrolls sideways inside its box; `phone: false` columns are
	// left out there.
	type Props = {
		/** What the rows are ("People"), for screen readers. */
		label: string;
		columns: TableColumn<Row>[];
		rows: Row[];
		rowKey: (row: Row) => string;
		cell: Snippet<[Row, TableColumn<Row>]>;
		sort?: TableSort;
		/** Quieter rows: revoked, hidden, switched off. */
		muted?: (row: Row) => boolean;
	};
	let { label, columns, rows, rowKey, cell, sort = $bindable(), muted }: Props = $props();

	const sorted = $derived.by(() => {
		const column = sort && columns.find((each) => each.key === sort!.key);
		if (!column?.sort) return rows;
		const value = column.sort;
		const way = sort!.direction === 'ascending' ? 1 : -1;
		return [...rows].sort((a, b) => {
			const left = value(a);
			const right = value(b);
			return (typeof left === 'number' && typeof right === 'number' ? left - right : String(left).localeCompare(String(right))) * way;
		});
	});

	function sortBy(key: string) {
		sort = { key, direction: sort?.key === key && sort.direction === 'ascending' ? 'descending' : 'ascending' };
	}
</script>

<div class="w-full overflow-x-auto rounded-md bg-bg outline -outline-offset-1 outline-border">
	<table class="w-full border-collapse">
		<caption class="sr-only">{label}</caption>
		<thead>
			<tr class="border-b border-border bg-surface">
				{#each columns as column (column.key)}
					{@const active = sort?.key === column.key}
					<th
						scope="col"
						aria-sort={column.sort ? (active ? sort!.direction : 'none') : undefined}
						class={[
							'px-1.5 pt-2.25 pb-2 font-mono text-label font-normal tracking-label whitespace-nowrap text-text-3 uppercase first:pl-3.5 last:pr-3.5',
							column.align === 'end' ? 'text-right' : 'text-left',
							column.phone === false && 'max-sm:hidden',
						]}
					>
						{#if column.sort}
							<button
								type="button"
								onclick={() => sortBy(column.key)}
								class={['inline-flex cursor-pointer items-center gap-1 rounded-xs tracking-label uppercase outline-offset-2 hover:text-text pointer-coarse:min-h-11', active ? 'text-text-2' : 'text-text-3']}
							>
								{column.label}
								<span aria-hidden="true" class="inline-flex"><Icon icon={active ? (sort!.direction === 'ascending' ? ArrowUp : ArrowDown) : ArrowUpDown} size={12} /></span>
							</button>
						{:else}
							{column.label}
						{/if}
					</th>
				{/each}
			</tr>
		</thead>
		<tbody>
			{#each sorted as row (rowKey(row))}
				<tr class={['border-b border-border last:border-b-0', muted?.(row) && 'opacity-60']}>
					{#each columns as column (column.key)}
						<td class={['px-1.5 py-2.25 align-middle text-ui text-text first:pl-3.5 last:pr-3.5', column.align === 'end' ? 'text-right' : 'text-left', column.phone === false && 'max-sm:hidden']}>
							{@render cell(row, column)}
						</td>
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>
