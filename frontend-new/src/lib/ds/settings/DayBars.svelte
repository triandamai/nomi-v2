<script lang="ts">
	// App / Settings / Plan & usage (C0og49) → Day by day: a bar a day across the month on a 140px
	// plot over its baseline, 3 apart, rounded at the top: past days in text-2, today in the accent,
	// days with nothing and the days ahead a 2px stub. The scale on the left (the top, half, 0), and
	// under it days 1, 7, 14, 21 and 28 in mono, with "today" under today in the text colour.
	export type DayBar = { day: number; value: number; share: number; state: 'past' | 'today' | 'future' | 'empty' };
	type Props = { label: string; bars: DayBar[]; scale: [string, string, string]; today: string; describe: (bar: DayBar) => string };
	let { label, bars, scale, today, describe }: Props = $props();

	const MARKS = [1, 7, 14, 21, 28];
	const todayAt = $derived(bars.find((bar) => bar.state === 'today')?.day);
	const mark = (day: number) => (day === todayAt ? today : MARKS.includes(day) && (!todayAt || Math.abs(day - todayAt) > 2) ? String(day) : '');
</script>

<figure aria-label={label} class="flex gap-3">
	<div aria-hidden="true" class="flex h-35 w-7 shrink-0 flex-col justify-between font-mono text-label text-text-3">
		{#each scale as tick, index (index)}<span>{tick}</span>{/each}
	</div>
	<div class="flex min-w-0 flex-1 flex-col gap-1.5">
		<ul class="flex h-35 items-end gap-0.75 border-b border-border">
			{#each bars as bar (bar.day)}
				<li class="flex h-full min-w-0 flex-1 items-end" title={describe(bar)}>
					<span class="sr-only">{describe(bar)}</span>
					{#if bar.state === 'past' || bar.state === 'today'}
						<span class={['w-full rounded-t-cell', bar.state === 'today' ? 'bg-accent' : 'bg-text-2']} style="height: {Math.max(2, Math.round(bar.share * 140))}px"></span>
					{:else}
						<span class="h-0.5 w-full bg-surface-2"></span>
					{/if}
				</li>
			{/each}
		</ul>
		<div aria-hidden="true" class="flex gap-0.75 font-mono text-label">
			{#each bars as bar (bar.day)}
				<span class={['relative flex min-w-0 flex-1 justify-center overflow-visible whitespace-nowrap', bar.day === todayAt ? 'text-text' : 'text-text-3']}>{mark(bar.day)}</span>
			{/each}
		</div>
	</div>
</figure>
