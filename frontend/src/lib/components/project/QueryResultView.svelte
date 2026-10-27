<script lang="ts">
	// What one SQL statement answered, in a project's data sheet: how many rows it changed, or
	// the rows it read as a grid.
	import { m } from '$lib/paraglide/messages';
	import type { QueryResult } from '$lib/types';

	let { result }: { result: QueryResult } = $props();

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
				{#each result.rows as row, r (r)}
					<tr>{#each row as value, c (c)}<td class:null={value === null}>{cell(value)}</td>{/each}</tr>
				{/each}
			</tbody>
		</table>
	</div>
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
	.null {
		color: var(--md-sys-color-on-surface-variant);
		font-style: italic;
	}
</style>
