<script lang="ts">
	import { Check, Copy, Download, Table2 } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import Icon from '../foundations/Icon.svelte';
	import IconButton from '../actions/IconButton.svelte';
	import Link from '../actions/Link.svelte';
	import { createCopier } from '../actions/copier.svelte';
	import { cellText, tableCsv, tableTsv, type Cell } from './table';

	// Chat responses → Table, for comparable rows only. The first column names the row; the others
	// are numbers, in mono and right-aligned. Where the data came from sits underneath.
	type Props = {
		title: string;
		columns: string[];
		rows: Cell[][];
		total?: Cell[];
		/** Units and origin, e.g. "Rp · from Dana · 142 transactions". */
		source?: string;
		link?: { href: string; label: string };
	};
	let { title, columns, rows, total, source, link }: Props = $props();

	const id = $props.id();
	const copier = createCopier();

	function download() {
		const url = URL.createObjectURL(new Blob([tableCsv(columns, rows, total)], { type: 'text/csv;charset=utf-8' }));
		const anchor = document.createElement('a');
		anchor.href = url;
		anchor.download = `${title.toLowerCase().replace(/[^\p{L}\p{N}]+/gu, '-').replace(/^-|-$/g, '') || 'table'}.csv`;
		anchor.click();
		URL.revokeObjectURL(url);
	}
</script>

{#snippet cell(value: Cell, index: number, isTotal: boolean)}
	<td
		class={[
			'py-2.25',
			index === 0 ? 'pr-1.5 pl-3.5 text-left text-ui' : 'w-27 px-1.5 text-right font-mono text-caption last:w-29 last:pr-3.5',
			index === 0 && (isTotal ? 'text-text' : 'text-text-2'),
			index > 0 && typeof value === 'string' && 'text-text',
			typeof value !== 'string' && value.tone === 'bad' && 'text-danger',
			typeof value !== 'string' && value.tone === 'good' && 'text-accent',
			typeof value !== 'string' && !value.tone && 'text-text',
		]}>{cellText(value)}</td
	>
{/snippet}

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2.5 border-b border-border bg-surface py-1 pr-1.5 pl-3.5">
		<span class="inline-flex text-text-2"><Icon icon={Table2} size={14} /></span>
		<p id="{id}-title" class="min-w-0 flex-1 truncate text-ui font-medium text-text">{title}</p>
		<IconButton icon={copier.copied ? Check : Copy} label={copier.copied ? m.copy_done() : m.table_copy()} variant="ghost" onclick={() => copier.copy(tableTsv(columns, rows, total))} />
		<IconButton icon={Download} label={m.table_download()} variant="ghost" onclick={download} />
	</header>
	<div class="overflow-x-auto">
		<table aria-labelledby="{id}-title" class="w-full border-collapse">
			<thead>
				<tr class="border-b border-border">
					{#each columns as column, index (index)}
						<th
							scope="col"
							class={[
								'py-2.25 font-mono text-label font-normal tracking-label whitespace-nowrap text-text-3 uppercase',
								index === 0 ? 'pr-1.5 pl-3.5 text-left' : 'w-27 px-1.5 text-right last:w-29 last:pr-3.5',
							]}>{column}</th
						>
					{/each}
				</tr>
			</thead>
			<tbody>
				{#each rows as row, rowIndex (rowIndex)}
					<tr>
						{#each row as value, index (index)}{@render cell(value, index, false)}{/each}
					</tr>
				{/each}
				{#if total}
					<tr class="border-t border-border bg-surface font-medium">
						{#each total as value, index (index)}{@render cell(value, index, true)}{/each}
					</tr>
				{/if}
			</tbody>
		</table>
	</div>
	{#if source || link}
		<footer class="flex items-center gap-2 border-t border-border px-3.5 py-2">
			<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">{source}</span>
			{#if link}<Link href={link.href} label={link.label} external quiet />{/if}
		</footer>
	{/if}
</section>
