<script lang="ts">
	// App / Settings / Profile (U9Gnrg) → Token Activity → Heatmap: a year of tokens, one 11px day
	// per cell, weeks as Monday-first columns 3 apart, shaded surface-2 then heat-1…4; Mon, Wed and
	// Fri named beside them in 9px mono, the months over their first week, and under a stroke the
	// note with the Less–More key. Scrolls sideways when narrower than the year, from its end.
	export type HeatDay = { date: string; tokens: number; level: 0 | 1 | 2 | 3 | 4 };
	type Props = {
		label: string;
		weeks: (HeatDay | null)[][];
		months: { label: string; week: number }[];
		/** Mon, Wed and Fri in the person's language (rows 1, 3 and 5). */
		days: [string, string, string];
		note?: string;
		less: string;
		more: string;
	};
	let { label, weeks, months, days, note, less, more }: Props = $props();

	const SHADES = ['bg-surface-2', 'bg-heat-1', 'bg-heat-2', 'bg-heat-3', 'bg-heat-4'] as const;
	// Narrower than a year (a phone), it opens on the latest weeks, which matter most.
	const toEnd = (scroller: HTMLElement) => void (scroller.scrollLeft = scroller.scrollWidth);
	const named = $derived(['', days[0], '', days[1], '', days[2], '', '']);
</script>

<figure class="flex flex-col gap-2 rounded-md px-4 py-3.5 outline -outline-offset-1 outline-border">
	<div class="overflow-x-auto" {@attach toEnd}>
		<div role="img" aria-label={label} class="flex w-max flex-col gap-2">
			<div class="flex gap-2">
				<div aria-hidden="true" class="flex w-6 flex-col gap-0.75">
					{#each named.slice(0, 7) as day, index (index)}<span class="h-2.75 font-mono text-nano text-text-3">{day}</span>{/each}
				</div>
				<div aria-hidden="true" class="flex gap-0.75">
					{#each weeks as week, index (index)}
						<div class="flex flex-col gap-0.75">
							{#each week as day, row (row)}
								{#if day}
									<span title="{day.date} · {day.tokens}" class={['size-2.75 rounded-cell', SHADES[day.level]]}></span>
								{:else}
									<span class="size-2.75"></span>
								{/if}
							{/each}
						</div>
					{/each}
				</div>
			</div>
			<div aria-hidden="true" class="relative h-3.5">
				{#each months as month (month.week)}
					<span class="absolute top-0 font-mono text-label text-text-3" style="left: {32 + month.week * 14}px">{month.label}</span>
				{/each}
			</div>
		</div>
	</div>
	<figcaption class="flex items-center gap-1.5 border-t border-border pt-1.25 font-mono text-label text-text-3">
		<span class="min-w-0 flex-1 truncate">{note}</span>
		<span>{less}</span>
		{#each SHADES as shade (shade)}<span aria-hidden="true" class={['size-2.5 rounded-cell', shade]}></span>{/each}
		<span>{more}</span>
	</figcaption>
</figure>
