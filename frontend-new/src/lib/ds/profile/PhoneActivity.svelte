<script lang="ts">
	import type { HeatDay } from './ActivityHeatmap.svelte';

	// Flow / Mobile — You & Settings, step 7 (Profile) → Stats and Heatmap: two numbers side by side
	// (surface, border, radius 14, 14 in: what it is at 13, the figure at 20 in mono, its note in
	// mono), then the last 13 weeks as a card: the total at 14, the weeks filling the width (cells
	// 18 high, 4 apart, radius 3, shaded surface-2 then heat-1…4, Monday at the top) and the busiest
	// day under them. A phone shows a quarter, not a year: a year at this width is too small to read.
	type Tile = { label: string; value: string; note: string };
	type Props = {
		tiles: Tile[];
		/** "412K tokens in the last 13 weeks": also the grid's name for a screen reader. */
		total: string;
		weeks: (HeatDay | null)[][];
		note?: string;
	};
	let { tiles, total, weeks, note }: Props = $props();

	const SHADES = ['bg-surface-2', 'bg-heat-1', 'bg-heat-2', 'bg-heat-3', 'bg-heat-4'] as const;
</script>

<div class="flex flex-col gap-3.5">
	{#if tiles.length}
		<dl class="flex gap-2.5">
			{#each tiles as tile (tile.label)}
				<div class="flex min-w-0 flex-1 flex-col gap-1 rounded-xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
					<dt class="text-caption text-text-3">{tile.label}</dt>
					<dd class="font-mono text-heading text-text">{tile.value}</dd>
					<dd class="font-mono text-label text-text-3">{tile.note}</dd>
				</div>
			{/each}
		</dl>
	{/if}
	<figure class="flex flex-col gap-2.5 rounded-xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
		<p class="text-phone-sm text-text">{total}</p>
		<div role="img" aria-label={total} class="flex gap-1">
			{#each weeks as week, index (index)}
				<div aria-hidden="true" class="flex min-w-0 flex-1 flex-col gap-1">
					{#each week as day, row (row)}
						{#if day}
							<span title="{day.date} · {day.tokens}" class={['h-4.5 rounded-2xs', SHADES[day.level]]}></span>
						{:else}
							<span class="h-4.5"></span>
						{/if}
					{/each}
				</div>
			{/each}
		</div>
		{#if note}<figcaption class="font-mono text-label text-text-3">{note}</figcaption>{/if}
	</figure>
</div>
