<script lang="ts">
	// Flow / Build — Workspace tools (U16wU) → Screen / Database (xYgNi) → Grid: a table's rows,
	// outlined: the column names in mono capitals on surface-2, then each row (32 high, [9,12], 12
	// apart) over its line, in mono. It scrolls across when it's wider than the sheet.
	type Props = { label: string; columns: string[]; rows: string[][] };
	let { label, columns, rows }: Props = $props();
</script>

<div class="overflow-x-auto rounded-sm outline -outline-offset-1 outline-border">
	<table aria-label={label} class="w-full border-collapse font-mono text-label">
		<thead>
			<tr class="h-8 bg-surface-2 text-text-3">
				{#each columns as column (column)}<th scope="col" class="px-3 text-left font-normal tracking-label whitespace-nowrap uppercase">{column}</th>{/each}
			</tr>
		</thead>
		<tbody>
			{#each rows as row, index (index)}
				<tr class="h-8 border-t border-border text-text">
					{#each row as cell, at (at)}<td class="max-w-80 truncate px-3 whitespace-nowrap">{cell}</td>{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>
