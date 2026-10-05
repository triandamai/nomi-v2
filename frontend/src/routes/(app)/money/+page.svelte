<script lang="ts">
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import { compareMonths, fillDays, formatAmount, monthLabel, niceScale, shiftMonth } from '$lib/money';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const money = $derived(data.money);
	const thisMonth = $derived(money?.months[0] ?? money?.month ?? '');
	const comparison = $derived(
		money ? compareMonths(money.total_cents, money.previous_total_cents, monthLabel(shiftMonth(money.month, -1), 'long').split(' ')[0]) : null,
	);
	const days = $derived(money ? fillDays(money.month, money.by_day) : []);
	const daysWithSpending = $derived(days.filter((d) => d.cents > 0).length);
	const dayScale = $derived(niceScale(Math.max(0, ...days.map((d) => d.cents)) / 100));
	const topCategory = $derived(money?.by_category[0] ?? null);
	const categoryMax = $derived(Math.max(1, ...(money?.by_category.map((c) => c.cents) ?? [1])));

	let query = $state('');
	let category = $state<string | null>(null);
	let showCategoryTable = $state(false);
	let showDayTable = $state(false);
	let hoverDay = $state<number | null>(null);
	let hoverCategory = $state<string | null>(null);

	const visibleTransactions = $derived.by(() => {
		if (!money) return [];
		const needle = query.trim().toLowerCase();
		return money.transactions.filter(
			(t) => (!category || t.category === category) && (!needle || t.description.toLowerCase().includes(needle)),
		);
	});

	function dayLabel(date: string): string {
		const [y, m, d] = date.split('-').map(Number);
		return new Intl.DateTimeFormat(undefined, { weekday: 'short', day: 'numeric', month: 'short', timeZone: 'UTC' }).format(
			new Date(Date.UTC(y, m - 1, d)),
		);
	}
	function txDate(iso: string): string {
		try {
			return new Intl.DateTimeFormat(undefined, { day: 'numeric', month: 'short', timeZone: money?.timezone }).format(new Date(iso));
		} catch {
			return iso.slice(0, 10);
		}
	}
	function share(cents: number): string {
		return money && money.total_cents > 0 ? `${Math.round((cents / money.total_cents) * 100)}%` : '';
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

<div class="money">
	<div class="money__inner">
		<header class="money__head">
			<div>
				<h1 class="md-display-small money__title">Money</h1>
				<p class="md-body-large money__lede">What you spent, where it went, and how this month compares. Money keeps an eye on it with you.</p>
			</div>
			<AgentShape agent="money" size={72} class="money__mark" />
		</header>

		{#if !money}
			<p class="money__notice" role="alert">Couldn't load your spending just now. Reload the page to try again.</p>
		{:else}
			<nav class="months" aria-label="Month">
				<a class="months__step" href="?month={shiftMonth(money.month, -1)}" aria-label="Previous month"><IconChevronLeft /></a>
				<span class="months__current">{monthLabel(money.month)}</span>
				{#if money.month < thisMonth}
					<a class="months__step" href="?month={shiftMonth(money.month, 1)}" aria-label="Next month"><IconChevronRight /></a>
				{:else}
					<span class="months__step months__step--off" aria-hidden="true"><IconChevronRight /></span>
				{/if}
			</nav>

			{#if money.transaction_count === 0}
				<section class="empty">
					<AgentShape agent="money" size={96} />
					<div>
						<h2 class="md-headline-small">No spending in {monthLabel(money.month)}</h2>
						<p class="md-body-large empty__body">When transactions come in, Money totals them here and spots where your money goes.</p>
					</div>
				</section>
			{:else}
				<section class="summary" aria-label="This month">
					<div class="hero">
						<span class="nomi-meta hero__label">Spent in {monthLabel(money.month, 'long').split(' ')[0]}</span>
						<span class="hero__value">{formatAmount(money.total_cents)}</span>
						{#if comparison}
							<span class="hero__delta" data-direction={comparison.direction}>
								<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
									{#if comparison.direction === 'up'}<path d="M12 19V5M5 12l7-7 7 7" />{:else if comparison.direction === 'down'}<path d="M12 5v14M19 12l-7 7-7-7" />{:else}<path d="M5 12h14" />{/if}
								</svg>
								{comparison.text}
							</span>
						{/if}
					</div>
					<dl class="tiles">
						<div class="tile">
							<dt>Transactions</dt>
							<dd>{money.transaction_count}</dd>
						</div>
						<div class="tile">
							<dt>Per spending day</dt>
							<dd>{formatAmount(daysWithSpending ? money.total_cents / daysWithSpending : 0)}</dd>
						</div>
						{#if topCategory}
							<div class="tile">
								<dt>Biggest category</dt>
								<dd class="tile__text">{topCategory.category}</dd>
							</div>
						{/if}
					</dl>
				</section>

				<div class="charts">
					<section class="panel" aria-labelledby="cat-heading">
						<div class="panel__head">
							<h2 id="cat-heading" class="panel__title">By category</h2>
							<button type="button" class="panel__toggle" aria-pressed={showCategoryTable} onclick={() => (showCategoryTable = !showCategoryTable)}>
								{showCategoryTable ? 'Chart' : 'Table'}
							</button>
						</div>
						{#if showCategoryTable}
							<table class="table">
								<thead><tr><th scope="col">Category</th><th scope="col">Transactions</th><th scope="col">Amount</th><th scope="col">Share</th></tr></thead>
								<tbody>
									{#each money.by_category as c (c.category)}
										<tr><td>{c.category}</td><td class="num">{c.count}</td><td class="num">{formatAmount(c.cents)}</td><td class="num">{share(c.cents)}</td></tr>
									{/each}
								</tbody>
							</table>
						{:else}
							<ul class="bars">
								{#each money.by_category as c (c.category)}
									<li>
										<button
											type="button"
											class="bars__row"
											class:bars__row--active={category === c.category}
											aria-pressed={category === c.category}
											aria-label="{c.category}: {formatAmount(c.cents)}, {share(c.cents)} of the month. Filter transactions"
											onclick={() => (category = category === c.category ? null : c.category)}
											onpointerenter={() => (hoverCategory = c.category)}
											onpointerleave={() => (hoverCategory = null)}
										>
											<span class="bars__label">{c.category}</span>
											<span class="bars__track">
												<span class="bars__fill" style="width: {Math.max(1.5, (c.cents / categoryMax) * 100)}%"></span>
												<span class="bars__value">{formatAmount(c.cents, { compact: true })}</span>
											</span>
											{#if hoverCategory === c.category}
												<span class="tooltip tooltip--bar" role="presentation">{c.count} {c.count === 1 ? 'transaction' : 'transactions'} · {share(c.cents)} of the month</span>
											{/if}
										</button>
									</li>
								{/each}
							</ul>
							<p class="panel__hint">Tap a category to filter the list below.</p>
						{/if}
					</section>

					<section class="panel" aria-labelledby="day-heading">
						<div class="panel__head">
							<h2 id="day-heading" class="panel__title">Day by day</h2>
							<button type="button" class="panel__toggle" aria-pressed={showDayTable} onclick={() => (showDayTable = !showDayTable)}>
								{showDayTable ? 'Chart' : 'Table'}
							</button>
						</div>
						{#if showDayTable}
							<div class="table-scroll">
								<table class="table">
									<thead><tr><th scope="col">Day</th><th scope="col">Amount</th></tr></thead>
									<tbody>
										{#each days.filter((d) => d.cents > 0) as d (d.date)}
											<tr><td>{dayLabel(d.date)}</td><td class="num">{formatAmount(d.cents)}</td></tr>
										{/each}
									</tbody>
								</table>
							</div>
						{:else}
							<div class="daychart">
								<svg viewBox="0 0 {CHART_W} {CHART_H}" class="daychart__svg" role="img" aria-label="Spending per day in {monthLabel(money.month)}; the table view lists each day">
									{#each dayScale.ticks as tick (tick)}
										<line x1={AXIS_W} x2={CHART_W} y1={y(tick)} y2={y(tick)} class="daychart__grid" />
										<text x={AXIS_W - 8} y={y(tick) + 4} class="daychart__tick" text-anchor="end">
											{new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(tick)}
										</text>
									{/each}
									{#each days as d (d.date)}
										{@const x = AXIS_W + (d.day - 1) * slot + (slot - barW) / 2}
										{#if d.cents > 0}
											<path d={barPath(x, barW, y(d.cents / 100))} class="daychart__bar" class:daychart__bar--dim={hoverDay !== null && hoverDay !== d.day} />
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
										style="left: {((AXIS_W + (hovered.day - 0.5) * slot) / CHART_W) * 100}%; top: {(y(hovered.cents / 100) / CHART_H) * 100}%"
									>
										<span class="tooltip__title">{dayLabel(hovered.date)}</span>
										<span>{hovered.cents > 0 ? formatAmount(hovered.cents) : 'No spending'}</span>
									</div>
								{/if}
							</div>
						{/if}
					</section>
				</div>

				<section class="panel" aria-labelledby="tx-heading">
					<div class="panel__head">
						<h2 id="tx-heading" class="panel__title">Transactions</h2>
						<span class="nomi-meta">{visibleTransactions.length} shown</span>
					</div>
					<div class="filters">
						<label class="search">
							<IconSearch />
							<span class="sr-only">Search transactions</span>
							<input type="search" bind:value={query} placeholder="Search descriptions" class="search__input" />
						</label>
						<div class="filters__chips">
							<Chip variant="filter" selected={category === null} onclick={() => (category = null)}>All</Chip>
							{#each money.by_category as c (c.category)}
								<Chip variant="filter" selected={category === c.category} onclick={() => (category = category === c.category ? null : c.category)}>
									{c.category}
								</Chip>
							{/each}
						</div>
					</div>
					{#if visibleTransactions.length === 0}
						<p class="money__notice">No transactions match.</p>
					{:else}
						<ul class="tx">
							{#each visibleTransactions as t (t.id)}
								<li class="tx__row">
									<span class="tx__date">{txDate(t.occurred_at)}</span>
									<span class="tx__desc">{t.description}</span>
									<span class="tx__cat">{t.category}</span>
									<span class="tx__amount">{formatAmount(t.amount_cents)}</span>
								</li>
							{/each}
						</ul>
					{/if}
				</section>
			{/if}
		{/if}
	</div>
</div>

<style>
	.money {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.money__inner {
		max-width: 1180px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 24px;
	}
	.money__head {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 24px;
	}
	.money__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.money__lede {
		margin: 8px 0 0;
		max-width: 60ch;
		color: var(--md-sys-color-on-surface-variant);
	}
	@media (max-width: 640px) {
		.money__head :global(.money__mark) {
			display: none;
		}
	}
	.money__notice {
		margin: 0;
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
	.months__step {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border-radius: var(--md-sys-shape-corner-full);
		color: var(--md-sys-color-on-surface);
	}
	a.months__step:hover {
		background: var(--md-sys-color-surface-container-highest);
	}
	.months__step--off {
		opacity: 0.3;
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

	.summary {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 320px), 1fr));
		gap: 16px;
		align-items: stretch;
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
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.tile dt {
		font-size: 0.875rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tile dd {
		margin: 0;
		font-size: 1.75rem;
		font-weight: 650;
	}
	.tile__text {
		text-transform: capitalize;
	}

	.charts {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 420px), 1fr));
		gap: 16px;
		align-items: start;
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
	.panel__toggle {
		height: 36px;
		padding: 0 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-full);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		font-size: 0.8125rem;
		font-weight: 600;
		cursor: pointer;
	}
	.panel__hint {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
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
	.bars__row {
		position: relative;
		display: grid;
		grid-template-columns: minmax(70px, 120px) minmax(0, 1fr);
		align-items: center;
		gap: 12px;
		width: 100%;
		min-height: 40px;
		padding: 4px 6px;
		border: none;
		border-radius: var(--md-sys-shape-corner-medium);
		background: transparent;
		color: inherit;
		font: inherit;
		text-align: left;
		cursor: pointer;
	}
	.bars__row:hover,
	.bars__row--active {
		background: var(--md-sys-color-surface-container);
	}
	.bars__row:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 1px;
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
	.tooltip--bar {
		right: 8px;
		top: -30px;
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

	.filters {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}
	.filters__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.search {
		display: flex;
		align-items: center;
		gap: 10px;
		max-width: 420px;
		height: 48px;
		padding: 0 18px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		color: var(--md-sys-color-on-surface-variant);
	}
	.search:focus-within {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.search__input {
		flex: 1;
		min-width: 0;
		border: none;
		outline: none;
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
	}
	.tx {
		list-style: none;
		margin: 0;
		padding: 0;
	}
	.tx__row {
		display: grid;
		grid-template-columns: 64px minmax(0, 1fr) auto auto;
		align-items: center;
		gap: 12px;
		padding: 12px 4px;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
		font-size: 0.9375rem;
	}
	.tx__row:last-child {
		border-bottom: none;
	}
	.tx__date {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.tx__desc {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.tx__cat {
		padding: 2px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-high);
		font-size: 0.75rem;
		font-weight: 600;
		text-transform: capitalize;
	}
	.tx__amount {
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		text-align: right;
	}
	@media (max-width: 560px) {
		.tx__row {
			grid-template-columns: 52px minmax(0, 1fr) auto;
		}
		.tx__cat {
			display: none;
		}
	}
</style>
