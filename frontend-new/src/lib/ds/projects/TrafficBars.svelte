<script lang="ts">
	// Flow / Build — Workspace tools (U16wU) → Screen / Monitor (EQNtT) → Traffic: requests per
	// bucket (a site's visits, Landing page WUiji), 96 high and 6 apart, bottom-aligned: each bar its
	// share of the busiest (text-2), a quiet bucket a 2px line in the border colour, an unmeasured one
	// nothing; the axis under it.
	type Bar = { at: string; share: number | null; count: number | null };
	type Props = { label: string; bars: Bar[]; axis: string[] };
	let { label, bars, axis }: Props = $props();
</script>

<div role="img" aria-label={label} class="flex flex-col gap-2">
	<div data-part="bars" class="flex h-24 items-end gap-1.5">
		{#each bars as bar (bar.at)}
			<span
				data-part="bar"
				title="{bar.at} · {bar.count ?? '—'}"
				style={bar.share ? `height: ${Math.max(2, Math.round(bar.share * 96))}px` : undefined}
				class={['min-w-0 flex-1 rounded-cell', bar.share === null ? 'h-0' : bar.share > 0 ? 'bg-text-2' : 'h-0.5 bg-border']}
			></span>
		{/each}
	</div>
	<div class="flex justify-between font-mono text-label text-text-3">
		{#each axis as tick (tick)}<span>{tick}</span>{/each}
	</div>
</div>
