<script lang="ts">
	import { tick } from 'svelte';

	// Flow / Build — Workspace tools (U16wU) → Screen / Database (xYgNi) → Grid: a table's rows,
	// outlined: the column names in mono capitals on surface-2, then each row (32 high, [9,12], 12
	// apart) over its line, in mono. It scrolls across when it's wider than the sheet.
	// With `edit` (the pen's Edit rows, whose editing it doesn't draw), each cell outside the key is a
	// button that opens a field of the same size in its place: Enter saves (the field stays open when
	// the save is refused), Esc puts the value back.
	type Props = {
		label: string;
		columns: string[];
		rows: string[][];
		edit?: {
			/** Columns that can't change here: the primary key's. */
			locked: number[];
			/** What the field starts with (a cell shown as a dash may hold nothing). */
			initial: (row: number, column: number) => string;
			/** Saves a cell; true once it's kept. */
			onsave: (row: number, column: number, value: string) => Promise<boolean>;
			/** "Edit status of row 3": names each cell's button and field. */
			cellLabel: (row: number, column: number) => string;
		};
	};
	let { label, columns, rows, edit }: Props = $props();

	let open = $state<{ row: number; column: number } | null>(null);
	let draft = $state('');
	let saving = $state(false);
	let field = $state<HTMLInputElement | null>(null);

	async function start(row: number, column: number) {
		if (!edit) return;
		open = { row, column };
		draft = edit.initial(row, column);
		await tick();
		field?.select();
	}
	async function save() {
		if (!edit || !open || saving) return;
		saving = true;
		const kept = await edit.onsave(open.row, open.column, draft);
		saving = false;
		if (kept) open = null;
	}
	function onkeydown(event: KeyboardEvent) {
		if (event.key === 'Enter') {
			event.preventDefault();
			void save();
		} else if (event.key === 'Escape') {
			event.preventDefault();
			event.stopPropagation();
			open = null;
		}
	}
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
					{#each row as cell, at (at)}
						{#if edit && !edit.locked.includes(at)}
							<td class="max-w-80 px-1.5 whitespace-nowrap">
								{#if open?.row === index && open.column === at}
									<input
										bind:this={field}
										bind:value={draft}
										aria-label={edit.cellLabel(index, at)}
										disabled={saving}
										{onkeydown}
										class="h-6 w-full min-w-24 rounded-xs border border-focus bg-bg px-1.25 font-mono text-label text-text outline-none"
									/>
								{:else}
									<button
										type="button"
										aria-label={edit.cellLabel(index, at)}
										onclick={() => void start(index, at)}
										class="block h-6 w-full max-w-77 cursor-text truncate rounded-xs px-1.5 text-left outline-offset-1 hover:bg-surface-2"
									>
										{cell}
									</button>
								{/if}
							</td>
						{:else}
							<td class="max-w-80 truncate px-3 whitespace-nowrap">{cell}</td>
						{/if}
					{/each}
				</tr>
			{/each}
		</tbody>
	</table>
</div>
