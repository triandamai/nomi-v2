<!-- Numbers at a glance, drawn in SVG at the bubble's own width: grouped bars, lines, or a donut.
     Each mark carries its value as a tooltip, and screen readers get the numbers as a table. -->
<script lang="ts">
	import { getLocale } from '$lib/paraglide/runtime';
	import ViewFrame from './ViewFrame.svelte';
	import { chartScale, formatTick, formatValue } from '$lib/views/format';
	import type { ChartView, ViewProps } from '$lib/views/types';

	let { view }: ViewProps<ChartView> = $props();

	// Series colours from the theme: the chart colour first, then the theme's other roles.
	const SERIES = ['var(--nomi-chart)', 'var(--md-sys-color-tertiary)', 'var(--md-sys-color-secondary)', 'var(--md-sys-color-error)', 'var(--md-sys-color-outline)', 'var(--md-sys-color-inverse-primary)'];

	let width = $state(320);
	const kind = $derived(view.chart ?? 'bar');
	const locale = getLocale();
	const show = (n: number) => formatValue(n, view.format, locale);

	// Bars and lines.
	const HEIGHT = 200;
	const PAD = { top: 8, right: 8, bottom: 28, left: 52 };
	const plotW = $derived(Math.max(60, width - PAD.left - PAD.right));
	const plotH = HEIGHT - PAD.top - PAD.bottom;
	const all = $derived(view.series.flatMap((s) => s.values));
	const low = $derived(Math.min(0, ...all));
	const ticks = $derived(chartScale(Math.max(...all, 0) - low).map((t) => t + low));
	const top = $derived(ticks.at(-1) ?? 1);
	const y = (n: number) => PAD.top + plotH - ((n - low) / (top - low || 1)) * plotH;
	const band = $derived(plotW / view.labels.length);
	const barW = $derived(Math.max(2, Math.min(28, (band * 0.72) / view.series.length)));
	const x = (i: number) => PAD.left + band * i + band / 2;
	// Leave out labels that wouldn't fit side by side.
	const labelEvery = $derived(Math.max(1, Math.ceil(view.labels.length / Math.max(1, Math.floor(plotW / 56)))));

	// Donut.
	const R = 70;
	const STROKE = 26;
	const total = $derived(view.series[0]?.values.reduce((a, b) => a + Math.max(0, b), 0) ?? 0);
	const slices = $derived.by(() => {
		let start = 0;
		const n = view.labels.length;
		return view.labels.map((label, i) => {
			const value = Math.max(0, view.series[0]?.values[i] ?? 0);
			const share = total > 0 ? value / total : 0;
			const slice = { label, value, share, start, color: `color-mix(in srgb, var(--nomi-chart) ${Math.round(100 - (i * 70) / Math.max(1, n - 1))}%, var(--md-sys-color-surface-container-lowest))` };
			start += share;
			return slice;
		});
	});
	const circumference = 2 * Math.PI * R;
</script>

<ViewFrame title={view.title}>
	<div class="chart" bind:clientWidth={width}>
		{#if kind === 'donut'}
			<div class="donut">
				<svg width="180" height="180" viewBox="0 0 180 180" role="img" aria-label={view.title ?? view.series[0]?.name}>
					<circle cx="90" cy="90" r={R} fill="none" stroke="var(--md-sys-color-surface-container-high)" stroke-width={STROKE} />
					{#each slices as slice, i (i)}
						{#if slice.share > 0}
							<circle
								cx="90"
								cy="90"
								r={R}
								fill="none"
								stroke={slice.color}
								stroke-width={STROKE}
								stroke-dasharray="{Math.max(0, slice.share * circumference - 2)} {circumference}"
								stroke-dashoffset={-slice.start * circumference}
								transform="rotate(-90 90 90)"
							><title>{slice.label}: {show(slice.value)}</title></circle>
						{/if}
					{/each}
					<text x="90" y="86" text-anchor="middle" class="donut__total">{formatTick(total, view.format, locale)}</text>
					<text x="90" y="106" text-anchor="middle" class="donut__caption">{view.series[0]?.name}</text>
				</svg>
				<ul class="legend legend--column">
					{#each slices as slice, i (i)}
						<li><span class="legend__swatch" style:background={slice.color}></span><span class="legend__name">{slice.label}</span><span class="legend__value">{Math.round(slice.share * 100)}%</span></li>
					{/each}
				</ul>
			</div>
		{:else}
			<svg {width} height={HEIGHT} role="img" aria-label={view.title ?? view.series.map((s) => s.name).join(', ')}>
				{#each ticks as tick (tick)}
					<line x1={PAD.left} x2={PAD.left + plotW} y1={y(tick)} y2={y(tick)} class="grid" class:grid--zero={tick === 0} />
					<text x={PAD.left - 8} y={y(tick)} dy="0.32em" text-anchor="end" class="tick">{formatTick(tick, view.format, locale)}</text>
				{/each}
				{#each view.labels as label, i (i)}
					{#if i % labelEvery === 0}
						<text x={x(i)} y={HEIGHT - 8} text-anchor="middle" class="tick">{label.length > 10 ? `${label.slice(0, 9)}…` : label}</text>
					{/if}
				{/each}
				{#if kind === 'bar'}
					{#each view.series as series, s (s)}
						{#each series.values as value, i (i)}
							{@const bx = x(i) - (barW * view.series.length) / 2 + s * barW}
							<rect x={bx + 1} y={Math.min(y(value), y(0))} width={barW - 2} height={Math.max(1, Math.abs(y(0) - y(value)))} rx={Math.min(4, (barW - 2) / 2)} fill={SERIES[s % SERIES.length]}>
								<title>{view.labels[i]} · {series.name}: {show(value)}</title>
							</rect>
						{/each}
					{/each}
				{:else}
					{#each view.series as series, s (s)}
						<polyline points={series.values.map((v, i) => `${x(i)},${y(v)}`).join(' ')} fill="none" stroke={SERIES[s % SERIES.length]} stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round" />
						{#each series.values as value, i (i)}
							<circle cx={x(i)} cy={y(value)} r={view.labels.length > 20 ? 0 : 3.5} fill={SERIES[s % SERIES.length]}>
								<title>{view.labels[i]} · {series.name}: {show(value)}</title>
							</circle>
						{/each}
					{/each}
				{/if}
			</svg>
			{#if view.series.length > 1}
				<ul class="legend">
					{#each view.series as series, s (s)}
						<li><span class="legend__swatch" style:background={SERIES[s % SERIES.length]}></span><span class="legend__name">{series.name}</span></li>
					{/each}
				</ul>
			{/if}
		{/if}

		<table class="sr-only">
			<thead><tr><th></th>{#each view.series as series, s (s)}<th>{series.name}</th>{/each}</tr></thead>
			<tbody>
				{#each view.labels as label, i (i)}
					<tr><th>{label}</th>{#each view.series as series, s (s)}<td>{show(series.values[i])}</td>{/each}</tr>
				{/each}
			</tbody>
		</table>
	</div>
</ViewFrame>

<style>
	.chart {
		position: relative;
		min-width: 0;
	}
	.chart svg {
		display: block;
		overflow: visible;
	}
	.grid {
		stroke: var(--md-sys-color-outline-variant);
		stroke-width: 1;
		stroke-dasharray: 2 4;
	}
	.grid--zero {
		stroke-dasharray: none;
	}
	.tick {
		fill: var(--md-sys-color-on-surface-variant);
		font-size: 11px;
		font-variant-numeric: tabular-nums;
	}
	.donut {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 16px 24px;
	}
	.donut__total {
		fill: var(--md-sys-color-on-surface);
		font-size: 18px;
		font-weight: 800;
	}
	.donut__caption {
		fill: var(--md-sys-color-on-surface-variant);
		font-size: 11px;
	}
	.legend {
		display: flex;
		flex-wrap: wrap;
		gap: 6px 16px;
		margin: 8px 0 0;
		padding: 0;
		list-style: none;
		font-size: 0.8125rem;
	}
	.legend--column {
		flex-direction: column;
		flex: 1;
		min-width: 140px;
		margin: 0;
	}
	.legend li {
		display: flex;
		align-items: center;
		gap: 8px;
	}
	.legend__swatch {
		flex: none;
		width: 10px;
		height: 10px;
		border-radius: 3px;
	}
	.legend__name {
		flex: 1;
		color: var(--md-sys-color-on-surface-variant);
	}
	.legend__value {
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
</style>
