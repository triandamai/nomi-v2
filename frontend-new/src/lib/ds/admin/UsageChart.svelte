<script lang="ts" module>
	export type ChartBar = {
		key: string;
		value: number;
		/** Shown under the bar where there's room ("1", "8", "15"); most bars have none. */
		tick?: string;
		/** What hovering or focusing the bar says: the day, then its figures. */
		tooltip: string[];
	};
</script>

<script lang="ts">
	// Admin → Usage chart: one measure over the days of a month (chat tokens), as thin bars on a zero
	// baseline, one neutral ink and no gradient: the figures are the point, not the colour. The scale
	// is honest: from zero to a round number above the largest day (`max`), with three quiet lines
	// and their values in mono. Hovering or focusing a bar shows its day and figures; arrow keys move
	// between bars. One series, so no legend: the title names it. The page offers the same figures
	// as a table.
	type Props = {
		title: string;
		bars: ChartBar[];
		/** The scale's top: a round number at or above the largest bar. */
		max: number;
		/** The scale's labels: top, middle, zero. */
		axis: [string, string, string];
	};
	let { title, bars, max, axis }: Props = $props();

	const id = $props.id();
	let shown = $state<string | null>(null);

	function onkeydown(event: KeyboardEvent, index: number) {
		const step = event.key === 'ArrowRight' ? 1 : event.key === 'ArrowLeft' ? -1 : 0;
		if (!step) return;
		event.preventDefault();
		const next = (event.currentTarget as HTMLElement).parentElement?.parentElement?.children[(index + step + bars.length) % bars.length]?.firstElementChild as HTMLElement | null;
		next?.focus();
	}
</script>

<figure aria-labelledby="{id}-title" class="flex w-full min-w-0 flex-col gap-3">
	<figcaption id="{id}-title" class="text-ui font-medium text-text">{title}</figcaption>
	<div class="flex w-full gap-2">
		<div aria-hidden="true" class="flex h-40 shrink-0 flex-col justify-between pb-5 text-right font-mono text-label text-text-3">
			<span class="-translate-y-1.5">{axis[0]}</span>
			<span>{axis[1]}</span>
			<span class="translate-y-1.5">{axis[2]}</span>
		</div>
		<div class="relative min-w-0 flex-1">
			<div aria-hidden="true" class="pointer-events-none absolute inset-x-0 top-0 bottom-5 flex flex-col justify-between">
				<span class="border-t border-border"></span>
				<span class="border-t border-border"></span>
				<span class="border-t border-border-strong"></span>
			</div>
			<ol class="relative flex h-40 items-stretch gap-0.5">
				{#each bars as bar, index (bar.key)}
					{@const height = max > 0 ? Math.min(bar.value / max, 1) * 100 : 0}
					<li class="relative flex min-w-0 flex-1 flex-col">
						<button
							type="button"
							aria-label={bar.tooltip.join(', ')}
							onfocus={() => (shown = bar.key)}
							onblur={() => (shown = null)}
							onmouseenter={() => (shown = bar.key)}
							onmouseleave={() => (shown = null)}
							onkeydown={(event) => onkeydown(event, index)}
							class="group flex min-h-0 flex-1 cursor-default items-end rounded-xs outline-offset-1"
						>
							<span
								data-bar
								class={['block w-full rounded-t-xs', shown === bar.key ? 'bg-text' : 'bg-text-3 group-hover:bg-text']}
								style="height: {height}%"
							></span>
						</button>
						<span aria-hidden="true" class="h-5 pt-1 text-center font-mono text-label whitespace-nowrap text-text-3">{bar.tick ?? ''}</span>
						{#if shown === bar.key}
							<span
								role="tooltip"
								class={[
									'absolute bottom-full z-10 mb-1 flex w-max flex-col gap-0.5 rounded-sm bg-surface px-2.5 py-1.5 shadow-float outline -outline-offset-1 outline-border-strong',
									index < bars.length / 2 ? 'left-0' : 'right-0',
								]}
							>
								<span class="text-caption font-medium text-text">{bar.tooltip[0]}</span>
								{#each bar.tooltip.slice(1) as line, at (at)}<span class="font-mono text-label text-text-2">{line}</span>{/each}
							</span>
						{/if}
					</li>
				{/each}
			</ol>
		</div>
	</div>
</figure>
