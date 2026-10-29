<script lang="ts">
	import type { Cell } from '../chat/table';

	// Mobile / Stat List: a table that fits a phone. Each row is its label, its total and the change
	// on one line, with the breakdown under it; the total row closes the list.
	type Delta = Exclude<Cell, string>;
	type Row = { label: string; value: string; delta?: Delta; detail?: string };
	type Props = { title: string; sub?: string; rows: Row[]; total?: Omit<Row, 'detail'> };
	let { title, sub, rows, total }: Props = $props();
</script>

{#snippet delta(change: Delta | undefined)}
	<span class={['w-10 shrink-0 text-right font-mono text-caption', change?.tone === 'bad' && 'text-danger', change?.tone === 'good' && 'text-accent', !change?.tone && 'text-text-3']}>{change?.value ?? ''}</span>
{/snippet}

<section class="flex w-full flex-col overflow-hidden rounded-md border border-border-strong bg-bg">
	<header class="flex items-center gap-2 border-b border-border px-3.5 py-3">
		<h3 class="min-w-0 flex-1 text-body font-semibold text-text">{title}</h3>
		{#if sub}<span class="font-mono text-label text-text-3">{sub}</span>{/if}
	</header>
	<ul>
		{#each rows as row, index (index)}
			<li class={['flex flex-col gap-0.75 px-3.5 py-2.5', index > 0 && 'border-t border-border']}>
				<span class="flex items-center gap-2">
					<span class="min-w-0 flex-1 text-body text-text">{row.label}</span>
					<span class="font-mono text-phone-sm text-text">{row.value}</span>
					{@render delta(row.delta)}
				</span>
				{#if row.detail}<span class="text-caption text-text-3">{row.detail}</span>{/if}
			</li>
		{/each}
		{#if total}
			<li class="flex items-center gap-2 border-t border-border bg-surface px-3.5 py-3">
				<span class="min-w-0 flex-1 text-body font-semibold text-text">{total.label}</span>
				<span class="font-mono text-phone-sm font-medium text-text">{total.value}</span>
				{@render delta(total.delta)}
			</li>
		{/if}
	</ul>
</section>
