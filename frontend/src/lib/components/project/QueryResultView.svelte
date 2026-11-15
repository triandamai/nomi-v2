<script lang="ts" module>
	/** Rows of one table can be edited in place when its primary key is known: `save` gets the
	 * row's key and the change, and answers with the row as it is now. */
	export interface RowEditing {
		primaryKey: string[];
		save: (key: Record<string, unknown>, changes: Record<string, unknown>) => Promise<Record<string, unknown>>;
	}
</script>

<script lang="ts">
	// What one SQL statement answered, in a project's data sheet: how many rows it changed, or
	// the rows it read as a grid.
	import { m } from '$lib/paraglide/messages';
	import type { QueryResult } from '$lib/types';
	import { cellText, parseCell, rowKey } from '$lib/rowEdit';

	let { result, editing = null }: { result: QueryResult; editing?: RowEditing | null } = $props();

	/** The cell being edited, its text, and why the last save failed. */
	let open = $state<{ row: number; column: number } | null>(null);
	let draft = $state('');
	let saving = $state(false);
	let editError = $state<string | null>(null);
	/** Rows as saved since this result came, by index; the result itself stays as it came. */
	let edited = $state<Record<number, unknown[]>>({});
	const rows = $derived(result.rows.map((row, i) => edited[i] ?? row));
	$effect(() => {
		void result;
		edited = {};
		open = null;
	});

	function canEdit(row: unknown[]): boolean {
		return !!editing && rowKey(result.columns, row, editing.primaryKey) !== null;
	}

	function start(row: number, column: number) {
		open = { row, column };
		draft = cellText(rows[row][column]);
		editError = null;
	}

	async function commit() {
		if (!open || !editing) return;
		const { row, column } = open;
		const original = rows[row][column];
		const key = rowKey(result.columns, rows[row], editing.primaryKey);
		if (!key) return;
		saving = true;
		editError = null;
		try {
			const name = result.columns[column];
			const now = await editing.save(key, { [name]: parseCell(draft, original) });
			const before = rows[row];
			edited[row] = result.columns.map((c, i) => (c in now ? now[c] : before[i]));
			open = null;
		} catch (e) {
			editError = (e as Error).message;
		} finally {
			saving = false;
		}
	}

	function keydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			void commit();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			open = null;
			editError = null;
		}
	}

	function cell(value: unknown): string {
		if (value === null || value === undefined) return 'NULL';
		return typeof value === 'object' ? JSON.stringify(value) : String(value);
	}
</script>

{#if result.rows_affected !== null}
	<p class="muted">{m.pj_db_affected({ count: result.rows_affected, ms: result.took_ms })}</p>
{:else}
	<p class="muted small">{m.pj_db_rows({ count: result.rows.length, ms: result.took_ms })}{result.truncated ? ` · ${m.pj_db_truncated()}` : ''}</p>
	<div class="grid-wrap">
		<table class="grid">
			<thead><tr>{#each result.columns as column, i (i)}<th>{column}</th>{/each}</tr></thead>
			<tbody>
				{#each rows as row, r (r)}
					<tr>
						{#each row as value, c (c)}
							{#if open && open.row === r && open.column === c}
								<td class="editing">
									<!-- svelte-ignore a11y_autofocus -->
									<input class="cell-input" aria-label={m.pj_db_edit_cell({ column: result.columns[c] })} bind:value={draft} onkeydown={keydown} onblur={() => !saving && (open = null)} disabled={saving} autofocus />
								</td>
							{:else if canEdit(row)}
								<td class:null={value === null}>
									<button type="button" class="cell" title={m.pj_db_edit_hint()} onclick={() => start(r, c)}>{cell(value)}</button>
								</td>
							{:else}
								<td class:null={value === null}>{cell(value)}</td>
							{/if}
						{/each}
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
	{#if editing}<p class="muted small">{m.pj_db_edit_hint()}</p>{/if}
	{#if editError}<p class="error" role="alert">{editError}</p>{/if}
{/if}

<style>
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.small {
		font-size: 0.8125rem;
	}
	.grid-wrap {
		max-height: 52vh;
		overflow: auto;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: 8px;
		background: var(--md-sys-color-surface-container-lowest);
	}
	.grid {
		border-collapse: collapse;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
	}
	.grid th,
	.grid td {
		max-width: 320px;
		padding: 6px 10px;
		overflow: hidden;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		text-align: left;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.grid th {
		position: sticky;
		top: 0;
		background: var(--md-sys-color-surface-container-high);
		font-weight: 700;
	}
	.cell {
		display: block;
		width: 100%;
		min-height: 24px;
		padding: 0;
		overflow: hidden;
		border: 0;
		background: none;
		color: inherit;
		font: inherit;
		text-align: left;
		text-overflow: ellipsis;
		cursor: text;
	}
	.cell:hover {
		text-decoration: underline dotted;
	}
	.cell:focus-visible,
	.cell-input:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
	}
	.editing {
		padding: 2px 4px;
	}
	.cell-input {
		width: 100%;
		min-width: 120px;
		padding: 4px 6px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: 4px;
		background: var(--md-sys-color-surface-container-low);
		color: inherit;
		font: inherit;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
	.null {
		color: var(--md-sys-color-on-surface-variant);
		font-style: italic;
	}
</style>
