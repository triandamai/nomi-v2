<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import ProSheet from '$lib/components/ProSheet.svelte';
	import PlanCard from '$lib/components/PlanCard.svelte';
	import Money from '$lib/components/Money.svelte';
	import { monthLabel, niceScale, shiftMonth } from '$lib/money';
	import { formatShare, formatTokens, formatTokensFull, formatUsd, usageShare } from '$lib/usage';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const usage = $derived(data.usage);
	const share = $derived(usage ? usageShare(usage.tokens_used, usage.plan.monthly_tokens) : 0);
	const isCurrent = $derived(usage ? usage.month === usage.current_month : true);
	let proOpen = $state(false);
	let showDayTable = $state(false);
	let hoverDay = $state<number | null>(null);

	// Every day of the month, with zero for quiet days.
	const days = $derived.by(() => {
		if (!usage) return [];
		const [year, monthNumber] = usage.month.split('-').map(Number);
		const count = new Date(Date.UTC(year, monthNumber, 0)).getUTCDate();
		const byDate = new Map(usage.by_day.map((d) => [d.date, d]));
		return Array.from({ length: count }, (_, i) => {
			const date = `${usage.month}-${String(i + 1).padStart(2, '0')}`;
			const found = byDate.get(date);
			return { date, day: i + 1, tokens: found?.tokens ?? 0, spend: found?.spend_usd ?? 0 };
		});
	});
	const dayScale = $derived(niceScale(Math.max(0, ...days.map((d) => d.tokens))));
	const modelMax = $derived(Math.max(1, ...(usage?.by_model.map((row) => row.tokens) ?? [1])));
	const resetsOn = $derived(
		usage
			? new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'long', timeZone: 'UTC' }).format(
					new Date(`${shiftMonth(usage.current_month, 1)}-01T00:00:00Z`),
				)
			: '',
	);

	function dayLabel(date: string): string {
		return new Intl.DateTimeFormat(getLocale(), { weekday: 'short', day: 'numeric', month: 'short', timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`));
	}

	// Day chart geometry (viewBox units).
	const CHART_W = 640;
	const CHART_H = 240;
	const AXIS_W = 48;
	const BASE_Y = CHART_H - 26;
	const TOP_Y = 10;
	const plotW = CHART_W - AXIS_W;
	const slot = $derived(days.length ? plotW / days.length : plotW);
	const barW = $derived(Math.min(24, Math.max(2, slot - 2)));
	function y(value: number): number {
		return BASE_Y - (value / dayScale.max) * (BASE_Y - TOP_Y);
	}
	function barPath(x: number, w: number, top: number): string {
		// Rounded 4px data end, square at the baseline.
		const r = Math.min(4, w / 2, BASE_Y - top);
		return `M${x} ${BASE_Y} V${top + r} Q${x} ${top} ${x + r} ${top} H${x + w - r} Q${x + w} ${top} ${x + w} ${top + r} V${BASE_Y} Z`;
	}
	const hovered = $derived(hoverDay !== null ? days[hoverDay - 1] : null);
</script>

<div class="billing">
	<div class="billing__inner">
		<header class="billing__head">
			<div class="billing__titles">
				<h1 class="md-display-small billing__title">{m.billing_title()}</h1>
				<p class="md-body-large billing__lede">{m.billing_lede()}</p>
			</div>
			<AgentShape agent="nomi" face size={64} class="billing__mark" />
		</header>

		{#if !usage}
			<p class="billing__notice" role="alert">{m.billing_load_failed()}</p>
		{:else}
			<nav class="months" aria-label={m.money_month()}>
				<IconButton variant="filled-tonal" href="?month={shiftMonth(usage.month, -1)}" aria-label={m.money_prev_month()}><IconChevronLeft /></IconButton>
				<span class="months__current">{monthLabel(usage.month)}</span>
				<IconButton
					variant="filled-tonal"
					href={isCurrent ? undefined : `?month=${shiftMonth(usage.month, 1)}`}
					disabled={isCurrent}
					aria-label={m.money_next_month()}
				>
					<IconChevronRight />
				</IconButton>
			</nav>

			<section class="summary" aria-label={m.billing_this_month()}>
				<div class="hero" class:hero--over={share >= 1}>
					<div class="hero__top">
						<span class="nomi-meta hero__label">{m.billing_used_this_month()}</span>
						<span class="plan-chip">{usage.plan.name}</span>
					</div>
					<span class="hero__value">{formatShare(share)}</span>
					<WavyProgress value={share} tone={share >= 0.9 ? 'ember' : 'glow'} label={m.usage_progress_label({ share: formatShare(share) })} />
					<span class="hero__delta">
						{m.usage_of_tokens({ used: formatTokens(usage.tokens_used), total: formatTokens(usage.plan.monthly_tokens) })}
						{#if isCurrent}· {m.billing_resets({ date: resetsOn })}{/if}
					</span>
					<div class="hero__actions">
						<Button variant="gradient" size="m" onclick={() => (proOpen = true)}>{m.usage_upgrade()}</Button>
					</div>
				</div>
				<dl class="tiles">
					<div class="tile">
						<dt>{m.billing_spend()}</dt>
						<dd><Money value={formatUsd(usage.spend_usd)} /></dd>
					</div>
					<div class="tile">
						<dt>{m.billing_input()}</dt>
						<dd title={formatTokensFull(usage.input_tokens)}>{formatTokens(usage.input_tokens)}</dd>
					</div>
					<div class="tile">
						<dt>{m.billing_output()}</dt>
						<dd title={formatTokensFull(usage.output_tokens)}>{formatTokens(usage.output_tokens)}</dd>
					</div>
					<div class="tile">
						<dt>{m.billing_requests()}</dt>
						<dd>{formatTokensFull(usage.calls)}</dd>
					</div>
					{#if usage.own_key_tokens > 0}
						<div class="tile">
							<dt>{m.billing_own_key()}</dt>
							<dd title={formatTokensFull(usage.own_key_tokens)}>{formatTokens(usage.own_key_tokens)}</dd>
						</div>
					{/if}
				</dl>
			</section>

			{#if usage.calls === 0}
				<section class="empty">
					<AgentShape agent="nomi" face size={88} />
					<div>
						<h2 class="md-headline-small">{m.billing_empty_title({ month: monthLabel(usage.month) })}</h2>
						<p class="md-body-large empty__body">{m.billing_empty_body()}</p>
					</div>
				</section>
			{:else}
				<div class="charts">
					<section class="panel" aria-labelledby="day-heading">
						<div class="panel__head">
							<h2 id="day-heading" class="panel__title">{m.billing_by_day()}</h2>
							<IconButton selected={showDayTable} aria-label={m.common_show_table()} onclick={() => (showDayTable = !showDayTable)}>
								<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M3 10h18M3 15h18M9 10v10" /></svg>
							</IconButton>
						</div>
						{#if showDayTable}
							<div class="table-scroll">
								<table class="table">
									<thead><tr><th scope="col">{m.money_day()}</th><th scope="col">{m.billing_tokens()}</th><th scope="col">{m.billing_spend()}</th></tr></thead>
									<tbody>
										{#each days.filter((d) => d.tokens > 0) as d (d.date)}
											<tr><td>{dayLabel(d.date)}</td><td class="num">{formatTokensFull(d.tokens)}</td><td class="num"><Money value={formatUsd(d.spend)} /></td></tr>
										{/each}
									</tbody>
								</table>
							</div>
						{:else}
							<div class="daychart">
								<svg viewBox="0 0 {CHART_W} {CHART_H}" class="daychart__svg" role="img" aria-label={m.billing_day_chart({ month: monthLabel(usage.month) })}>
									{#each dayScale.ticks as tick (tick)}
										<line x1={AXIS_W} x2={CHART_W} y1={y(tick)} y2={y(tick)} class="daychart__grid" />
										<text x={AXIS_W - 8} y={y(tick) + 4} class="daychart__tick" text-anchor="end">{formatTokens(tick)}</text>
									{/each}
									{#each days as d (d.date)}
										{@const x = AXIS_W + (d.day - 1) * slot + (slot - barW) / 2}
										{#if d.tokens > 0}
											<path d={barPath(x, barW, y(d.tokens))} class="daychart__bar" class:daychart__bar--dim={hoverDay !== null && hoverDay !== d.day} />
										{/if}
										{#if d.day === 1 || d.day % 7 === 0}
											<text x={x + barW / 2} y={CHART_H - 4} class="daychart__tick" text-anchor="middle">{d.day}</text>
										{/if}
										<rect
											x={AXIS_W + (d.day - 1) * slot}
											y={TOP_Y}
											width={slot}
											height={BASE_Y - TOP_Y}
											class="daychart__hit"
											role="presentation"
											onpointerenter={() => (hoverDay = d.day)}
											onpointerleave={() => (hoverDay = null)}
										/>
									{/each}
								</svg>
								{#if hovered}
									<div
										class="tooltip tooltip--day"
										style="left: {((AXIS_W + (hovered.day - 0.5) * slot) / CHART_W) * 100}%; top: {(y(hovered.tokens) / CHART_H) * 100}%"
									>
										<span class="tooltip__title">{dayLabel(hovered.date)}</span>
										<span>{#if hovered.tokens > 0}<Money value={m.billing_day_tip({ tokens: formatTokensFull(hovered.tokens), spend: formatUsd(hovered.spend) })} />{:else}{m.billing_no_use()}{/if}</span>
									</div>
								{/if}
							</div>
						{/if}
					</section>

					<section class="panel" aria-labelledby="model-heading">
						<div class="panel__head">
							<h2 id="model-heading" class="panel__title">{m.billing_by_model()}</h2>
						</div>
						<ul class="bars models">
							{#each usage.by_model as row (row.label + row.model_id + row.source)}
								<li class="model">
									<div class="model__top">
										<span class="bars__label">{row.label}</span>
										{#if row.source === 'own_key'}<span class="key-chip">{m.billing_your_key()}</span>{/if}
										<span class="model__spend">{#if row.source === 'own_key'}{m.billing_not_billed()}{:else}<Money value={formatUsd(row.spend_usd)} />{/if}</span>
									</div>
									<span class="bars__track">
										<span class="bars__fill" style="width: {Math.max(1.5, (row.tokens / modelMax) * 100)}%"></span>
										<span class="bars__value">{formatTokens(row.tokens)}</span>
									</span>
									<span class="model__meta">{row.provider} · {row.model_id} · {row.calls === 1 ? m.billing_requests_one() : m.billing_requests_count({ count: formatTokensFull(row.calls) })}</span>
								</li>
							{/each}
						</ul>
					</section>
				</div>
			{/if}

			<section class="plans" aria-labelledby="plans-heading">
				<h2 id="plans-heading" class="section-label">{m.billing_plans()}</h2>
				{#if data.plans}
					<div class="plans__grid">
						{#each data.plans.plans as plan (plan.id)}
							<PlanCard {plan} current={plan.id === data.plans.current_plan_id} />
						{/each}
					</div>
					{#if data.plans.custom_quota}
						<p class="billing__note">{m.billing_custom_quota({ tokens: formatTokensFull(data.plans.monthly_tokens) })}</p>
					{/if}
				{/if}
				<p class="billing__note">{m.billing_note()}</p>
			</section>
		{/if}
	</div>
</div>

<ProSheet bind:open={proOpen} />

<style>
	.billing {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.billing__inner {
		max-width: 1180px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 24px;
	}
	.billing__head {
		display: flex;
		align-items: flex-end;
		justify-content: space-between;
		gap: 20px;
	}
	.billing__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.billing__lede {
		margin: 8px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	@media (max-width: 640px) {
		.billing__head :global(.billing__mark) {
			display: none;
		}
	}
	.billing__notice,
	.billing__note {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.billing__note {
		font-size: 0.875rem;
		max-width: 70ch;
	}
	.summary {
		display: grid;
		grid-template-columns: minmax(0, 1.2fr) minmax(0, 1fr);
		gap: 16px;
		align-items: stretch;
	}
	@media (max-width: 860px) {
		.summary {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	.hero__top {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.hero--over .hero__value {
		color: var(--md-sys-color-error-container);
	}
	.hero__actions {
		margin-top: 8px;
	}
	.plan-chip {
		padding: 4px 12px;
		border-radius: 999px;
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
		font-size: 0.75rem;
		font-weight: 700;
	}
	.charts {
		display: grid;
		grid-template-columns: minmax(0, 1.4fr) minmax(0, 1fr);
		gap: 16px;
	}
	@media (max-width: 960px) {
		.charts {
			grid-template-columns: minmax(0, 1fr);
		}
	}
	/* On a phone the chart keeps a readable size and scrolls sideways in its box. */
	@media (max-width: 600px) {
		.daychart {
			overflow-x: auto;
			padding-top: 56px;
			margin-top: -56px;
		}
		.daychart__svg {
			min-width: 540px;
		}
	}
	.model {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.model__top {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.bars.models {
		gap: 20px;
	}
	/* Model names are names: keep them exactly as written. */
	.model__top .bars__label {
		flex: 1 1 auto;
		min-width: 0;
		text-transform: none;
	}
	.model__spend {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface);
	}
	.model__meta {
		font-size: 0.75rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.key-chip {
		padding: 2px 8px;
		border-radius: 999px;
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-size: 0.6875rem;
		font-weight: 700;
	}
	.plans {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}
	.plans__grid {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
		gap: 16px;
	}
	.section-label {
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.75rem;
		letter-spacing: 0.08em;
		text-transform: uppercase;
		color: var(--md-sys-color-on-surface-variant);
	}
	.months {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		gap: 4px;
		padding: 4px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
	}
	.months__current {
		min-width: 140px;
		text-align: center;
		font-weight: 650;
	}
	.empty {
		display: flex;
		align-items: center;
		gap: 28px;
		flex-wrap: wrap;
		padding: clamp(24px, 4vw, 48px);
		border-radius: var(--md-sys-shape-corner-extra-extra-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
	}
	.empty h2 {
		margin: 0;
	}
	.empty__body {
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.hero {
		display: flex;
		flex-direction: column;
		gap: 8px;
		padding: 28px;
		border-radius: 40px 40px 40px 12px;
		background: var(--nomi-color-stage);
		color: var(--nomi-color-on-stage);
	}
	.hero__label {
		color: inherit;
		opacity: 0.75;
	}
	/* The hero figure stays in the body sans, proportional figures (dataviz: hero figure). */
	.hero__value {
		font-family: var(--md-sys-typescale-body-large-font);
		font-size: clamp(2.75rem, 7vw, 4rem);
		font-weight: 650;
		line-height: 1.05;
		letter-spacing: -0.02em;
	}
	.hero__delta {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 0.9375rem;
		font-weight: 600;
	}
	.tiles {
		margin: 0;
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
		gap: 12px;
	}
	.tile {
		display: flex;
		flex-direction: column;
		justify-content: center;
		gap: 6px;
		min-width: 0;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		/* The value scales with the tile, so a long category or amount stays inside it. */
		container-type: inline-size;
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.tile dt {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tile dd {
		margin: 0;
		font-size: clamp(1.125rem, 15cqi, 1.75rem);
		font-weight: 650;
		line-height: 1.2;
		overflow-wrap: anywhere;
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 14px;
		padding: 22px 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		min-width: 0;
	}
	.panel__head {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
	}
	.panel__title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 700;
	}
	/* Category bars: one hue, ≤24px thick, 4px rounded data end, value at the tip. */
	.bars {
		list-style: none;
		margin: 0;
		padding: 0;
		display: flex;
		flex-direction: column;
		gap: 2px;
	}
	.bars__label {
		font-size: 0.875rem;
		font-weight: 600;
		text-transform: capitalize;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.bars__track {
		display: flex;
		align-items: center;
		gap: 8px;
		min-width: 0;
	}
	.bars__fill {
		height: 20px;
		border-radius: 0 4px 4px 0;
		background: var(--nomi-chart-money);
	}
	.bars__value {
		flex: none;
		font-size: 0.8125rem;
		font-variant-numeric: tabular-nums;
		color: var(--md-sys-color-on-surface);
	}
	.daychart {
		position: relative;
	}
	.daychart__svg {
		display: block;
		width: 100%;
		height: auto;
		overflow: visible;
	}
	.daychart__grid {
		stroke: var(--md-sys-color-outline-variant);
		stroke-width: 1;
		vector-effect: non-scaling-stroke;
	}
	.daychart__tick {
		font-size: 15px;
		fill: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
	.daychart__bar {
		fill: var(--nomi-chart-money);
		transition: opacity var(--nomi-motion-effects-fast);
	}
	.daychart__bar--dim {
		opacity: 0.35;
	}
	.daychart__hit {
		fill: transparent;
	}
	.tooltip {
		position: absolute;
		z-index: 2;
		display: flex;
		flex-direction: column;
		gap: 2px;
		padding: 8px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-inverse-surface);
		color: var(--md-sys-color-inverse-on-surface);
		font-size: 0.8125rem;
		white-space: nowrap;
		pointer-events: none;
	}
	.tooltip__title {
		font-weight: 650;
	}
	/* Sits just above the hovered bar's top. */
	.tooltip--day {
		translate: -50% calc(-100% - 8px);
	}
	.table-scroll {
		max-height: 260px;
		overflow-y: auto;
	}
	.table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}
	.table th {
		text-align: left;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
		padding: 8px 6px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.table td {
		padding: 8px 6px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		text-transform: capitalize;
	}
	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
	}
</style>
