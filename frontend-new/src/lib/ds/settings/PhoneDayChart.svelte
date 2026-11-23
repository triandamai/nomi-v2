<script lang="ts">
	import type { DayBar } from './DayBars.svelte';

	// Flow / Mobile — You & Settings, step 8 (Plan & usage) → Chart: a bar a day up to today across
	// the card's width (surface, border, radius 14, 14 in, 120 high), 3 apart with radius 2 tops:
	// the days in heat-2, today in the accent; the days still to come aren't drawn. Each bar says its
	// day and amount to a screen reader.
	type Props = { label: string; bars: DayBar[]; describe: (bar: DayBar) => string };
	let { label, bars, describe }: Props = $props();

	const shown = $derived(bars.filter((bar) => bar.state !== 'future'));
</script>

<figure class="flex h-30 flex-col rounded-xl bg-surface p-3.5 outline -outline-offset-1 outline-border">
	<ul aria-label={label} class="flex min-h-0 flex-1 items-end gap-0.75">
		{#each shown as bar (bar.day)}
			<li class="flex h-full min-w-0 flex-1 items-end" title={describe(bar)}>
				<span class="sr-only">{describe(bar)}</span>
				<span class={['w-full rounded-t-cell', bar.state === 'today' ? 'bg-accent' : 'bg-heat-2']} style="height: max(2px, {Math.round(bar.share * 100)}%)"></span>
			</li>
		{/each}
	</ul>
</figure>
