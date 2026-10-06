<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import AgentShape from '$lib/components/m3/AgentShape.svelte';
	import { enhance } from '$app/forms';
	import BottomSheet from '$lib/components/m3/BottomSheet.svelte';
	import Button from '$lib/components/m3/Button.svelte';
	import Chip from '$lib/components/m3/Chip.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import List from '$lib/components/m3/List.svelte';
	import ListItem from '$lib/components/m3/ListItem.svelte';
	import Snackbar from '$lib/components/m3/Snackbar.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import WavyProgress from '$lib/components/m3/WavyProgress.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import IconPlus from '$lib/components/icons/IconPlus.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import IconSearch from '$lib/components/icons/IconSearch.svelte';
	import { budgetState, categoryLook, compareMonths, fillDays, formatAmount, monthLabel, monthName, niceScale, shiftMonth } from '$lib/money';
	import { getLocale } from '$lib/paraglide/runtime';
	import type { ActionData, PageData } from './$types';

	let { data, form }: { data: PageData; form: ActionData } = $props();

	// Bottom sheets for the page's two edits, and the snackbar that confirms them.
	let expenseOpen = $state(false);
	let budgetOpen = $state(false);
	let budgetCategory = $state('');
	let budgetLimit = $state('');
	let saving = $state(false);
	let snackbar = $state(false);
	let snackbarMessage = $state('');
	const today = new Date().toISOString().slice(0, 10);

	function openBudget(category = '', limitCents?: number) {
		budgetCategory = category;
		budgetLimit = limitCents ? String(limitCents / 100) : '';
		budgetOpen = true;
	}
	function notify(message: string) {
		snackbarMessage = message;
		snackbar = true;
	}
	function sheetSubmit(close: () => void, message: string) {
		return () => {
			saving = true;
			return async ({ result, update }: { result: { type: string }; update: () => Promise<void> }) => {
				await update();
				saving = false;
				if (result.type === 'success') {
					close();
					notify(message);
				}
			};
		};
	}
	const knownCategories = $derived(
		[...new Set([...(data.money?.by_category.map((c) => c.category) ?? []), ...(data.money?.budgets.map((b) => b.category) ?? [])])].sort(),
	);

	const money = $derived(data.money);
	const thisMonth = $derived(money?.months[0] ?? money?.month ?? '');
	const comparison = $derived(
		money ? compareMonths(money.total_cents, money.previous_total_cents, monthName(shiftMonth(money.month, -1))) : null,
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
		const [y, mo, d] = date.split('-').map(Number);
		return new Intl.DateTimeFormat(getLocale(), { weekday: 'short', day: 'numeric', month: 'short', timeZone: 'UTC' }).format(
			new Date(Date.UTC(y, mo - 1, d)),
		);
	}
	function txDate(iso: string): string {
		try {
			return new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'short', timeZone: money?.timezone }).format(new Date(iso));
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
			<div class="money__titles">
				<h1 class="md-display-small money__title">{m.money_title()}</h1>
				<p class="md-body-large money__lede">{m.money_lede()}</p>
			</div>
			<div class="money__actions">
				<AgentShape agent="money" size={64} class="money__mark" />
				<Button variant="tonal" size="m" onclick={() => openBudget()}>{m.money_set_budget()}</Button>
				<Button variant="gradient" size="m" onclick={() => (expenseOpen = true)}>
					<IconPlus size={20} />
					{m.money_add_expense()}
				</Button>
			</div>
		</header>

		{#if !money}
			<p class="money__notice" role="alert">{m.money_load_failed()}</p>
		{:else}
			<nav class="months" aria-label={m.money_month()}>
				<IconButton variant="filled-tonal" href="?month={shiftMonth(money.month, -1)}" aria-label={m.money_prev_month()}><IconChevronLeft /></IconButton>
				<span class="months__current">{monthLabel(money.month)}</span>
				<IconButton
					variant="filled-tonal"
					href={money.month < thisMonth ? `?month=${shiftMonth(money.month, 1)}` : undefined}
					disabled={money.month >= thisMonth}
					aria-label={m.money_next_month()}
				>
					<IconChevronRight />
				</IconButton>
			</nav>

			{#if money.transaction_count === 0}
				<section class="empty">
					<AgentShape agent="money" size={96} />
					<div>
						<h2 class="md-headline-small">{m.money_empty_title({ month: monthLabel(money.month) })}</h2>
						<p class="md-body-large empty__body">{m.money_empty_body()}</p>
					</div>
				</section>
			{:else}
				<section class="summary" aria-label={m.money_this_month()}>
					<div class="hero">
						<span class="nomi-meta hero__label">{m.money_spent_in({ month: monthName(money.month) })}</span>
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
							<dt>{m.money_transactions()}</dt>
							<dd>{money.transaction_count}</dd>
						</div>
						<div class="tile">
							<dt>{m.money_per_day()}</dt>
							<dd>{formatAmount(daysWithSpending ? money.total_cents / daysWithSpending : 0)}</dd>
						</div>
						{#if topCategory}
							<div class="tile">
								<dt>{m.money_biggest()}</dt>
								<dd class="tile__text">{topCategory.category}</dd>
							</div>
						{/if}
					</dl>
				</section>

				<section class="panel" aria-labelledby="budget-heading">
					<div class="panel__head">
						<h2 id="budget-heading" class="panel__title">{m.money_budgets()}</h2>
						<Button variant="text" onclick={() => openBudget()}>{m.common_add()}</Button>
					</div>
					{#if money.budgets.length === 0}
						<p class="panel__hint">{m.money_budgets_hint()}</p>
					{:else}
						<ul class="budgets">
							{#each money.budgets as b (b.category)}
								{@const look = categoryLook(b.category)}
								{@const state = budgetState(b.spent_cents, b.limit_cents)}
								<li class="budget" class:budget--over={state.over}>
									<div class="budget__top">
										<AgentShape shape={look.shape} tone={look.tone} size={36} />
										<span class="budget__name">{b.category}</span>
										<IconButton aria-label={m.money_edit_budget({ category: b.category })} onclick={() => openBudget(b.category, b.limit_cents)}>
											<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 20h9M16.5 3.5a2.1 2.1 0 0 1 3 3L7 19l-4 1 1-4Z" /></svg>
										</IconButton>
										<form
											method="POST"
											action="?/deleteBudget"
											use:enhance={() => async ({ result, update }) => {
												await update();
												if (result.type === 'success') notify(m.money_budget_removed({ category: b.category }));
											}}
										>
											<input type="hidden" name="category" value={b.category} />
											<IconButton type="submit" aria-label={m.money_remove_budget({ category: b.category })}><IconClose size={18} /></IconButton>
										</form>
									</div>
									<div class="budget__row">
										<span class="budget__figures">{formatAmount(b.spent_cents)} <span class="budget__of">{m.money_of({ amount: formatAmount(b.limit_cents) })}</span></span>
										{#if state.over}
											<span class="budget__flag">
												<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12 9v4M12 17h.01M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z" /></svg>
												{m.money_over_by({ amount: formatAmount(b.spent_cents - b.limit_cents) })}
											</span>
										{:else if state.nearly}
											<span class="budget__flag budget__flag--near">{m.money_left({ amount: formatAmount(b.limit_cents - b.spent_cents) })}</span>
										{:else}
											<span class="budget__left">{m.money_used({ percent: Math.round(state.ratio * 100) })}</span>
										{/if}
									</div>
									<WavyProgress value={Math.min(1, state.ratio)} tone={look.tone} label={m.money_budget_progress({ category: b.category, percent: Math.round(state.ratio * 100) })} />
								</li>
							{/each}
						</ul>
					{/if}
				</section>

				<div class="charts">
					<section class="panel" aria-labelledby="cat-heading">
						<div class="panel__head">
							<h2 id="cat-heading" class="panel__title">{m.money_by_category()}</h2>
							<IconButton selected={showCategoryTable} aria-label={m.common_show_table()} onclick={() => (showCategoryTable = !showCategoryTable)}>
								<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M3 10h18M3 15h18M9 10v10" /></svg>
							</IconButton>
						</div>
						{#if showCategoryTable}
							<table class="table">
								<thead><tr><th scope="col">{m.common_category()}</th><th scope="col">{m.money_transactions()}</th><th scope="col">{m.common_amount()}</th><th scope="col">{m.money_share()}</th></tr></thead>
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
											aria-label={m.money_bar_label({ category: c.category, amount: formatAmount(c.cents), share: share(c.cents) })}
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
												<span class="tooltip tooltip--bar" role="presentation">{c.count === 1 ? m.money_tx_one() : m.money_tx_many({ count: c.count })} · {m.money_of_month({ share: share(c.cents) })}</span>
											{/if}
										</button>
									</li>
								{/each}
							</ul>
							<p class="panel__hint">{m.money_tap_category()}</p>
						{/if}
					</section>

					<section class="panel" aria-labelledby="day-heading">
						<div class="panel__head">
							<h2 id="day-heading" class="panel__title">{m.money_day_by_day()}</h2>
							<IconButton selected={showDayTable} aria-label={m.common_show_table()} onclick={() => (showDayTable = !showDayTable)}>
								<svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><rect x="3" y="4" width="18" height="16" rx="2" /><path d="M3 10h18M3 15h18M9 10v10" /></svg>
							</IconButton>
						</div>
						{#if showDayTable}
							<div class="table-scroll">
								<table class="table">
									<thead><tr><th scope="col">{m.money_day()}</th><th scope="col">{m.common_amount()}</th></tr></thead>
									<tbody>
										{#each days.filter((d) => d.cents > 0) as d (d.date)}
											<tr><td>{dayLabel(d.date)}</td><td class="num">{formatAmount(d.cents)}</td></tr>
										{/each}
									</tbody>
								</table>
							</div>
						{:else}
							<div class="daychart">
								<svg viewBox="0 0 {CHART_W} {CHART_H}" class="daychart__svg" role="img" aria-label={m.money_day_chart({ month: monthLabel(money.month) })}>
									{#each dayScale.ticks as tick (tick)}
										<line x1={AXIS_W} x2={CHART_W} y1={y(tick)} y2={y(tick)} class="daychart__grid" />
										<text x={AXIS_W - 8} y={y(tick) + 4} class="daychart__tick" text-anchor="end">
											{new Intl.NumberFormat(getLocale(), { notation: 'compact', maximumFractionDigits: 1 }).format(tick)}
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
										<span>{hovered.cents > 0 ? formatAmount(hovered.cents) : m.money_no_spending()}</span>
									</div>
								{/if}
							</div>
						{/if}
					</section>
				</div>

				<section class="panel" aria-labelledby="tx-heading">
					<div class="panel__head">
						<h2 id="tx-heading" class="panel__title">{m.money_transactions()}</h2>
						<span class="nomi-meta">{m.money_shown({ count: visibleTransactions.length })}</span>
					</div>
					<div class="filters">
						<label class="search">
							<IconSearch />
							<span class="sr-only">{m.money_search()}</span>
							<input type="search" bind:value={query} placeholder={m.money_search_placeholder()} class="search__input" />
						</label>
						<div class="filters__chips">
							<Chip variant="filter" selected={category === null} onclick={() => (category = null)}>{m.common_all()}</Chip>
							{#each money.by_category as c (c.category)}
								<Chip variant="filter" selected={category === c.category} onclick={() => (category = category === c.category ? null : c.category)}>
									{c.category}
								</Chip>
							{/each}
						</div>
					</div>
					{#if visibleTransactions.length === 0}
						<p class="money__notice">{m.money_no_match()}</p>
					{:else}
						<List class="tx">
							{#each visibleTransactions as t (t.id)}
								{@const look = categoryLook(t.category)}
								<ListItem headline={t.description} supportingText="{t.category} · {txDate(t.occurred_at)}">
									{#snippet leading()}
										<AgentShape shape={look.shape} tone={look.tone} size={36} />
									{/snippet}
									{#snippet trailing()}
										<span class="tx__amount">{formatAmount(t.amount_cents)}</span>
									{/snippet}
								</ListItem>
							{/each}
						</List>
					{/if}
				</section>
			{/if}
		{/if}
	</div>
</div>

<BottomSheet bind:open={expenseOpen}>
	<form method="POST" action="?/addExpense" class="sheet" use:enhance={sheetSubmit(() => (expenseOpen = false), m.money_expense_added())}>
		<div class="sheet__head">
			<AgentShape agent="money" size={36} />
			<h2 class="md-headline-small-emphasized sheet__title">{m.money_add_expense_title()}</h2>
		</div>
		<TextField id="expense-amount" name="amount" label={m.common_amount()} type="number" inputmode="decimal" min="0.01" step="0.01" required />
		<TextField id="expense-description" name="description" label={m.money_what_for()} required maxlength={200} />
		<TextField id="expense-category" name="category" label={m.common_category()} required maxlength={40} list="money-categories" />
		<datalist id="money-categories">
			{#each knownCategories as c (c)}<option value={c}></option>{/each}
		</datalist>
		<label class="sheet__field">
			<span class="sheet__label">{m.common_date()}</span>
			<input type="date" name="occurred_at" value={today} max={today} class="sheet__date" />
		</label>
		{#if form?.error}<p class="sheet__error" role="alert">{form.error}</p>{/if}
		<div class="sheet__actions">
			<Button type="button" variant="text" onclick={() => (expenseOpen = false)}>{m.common_cancel()}</Button>
			<Button type="submit" variant="filled" size="m" disabled={saving}>{m.money_add_expense()}</Button>
		</div>
	</form>
</BottomSheet>

<BottomSheet bind:open={budgetOpen}>
	<form method="POST" action="?/setBudget" class="sheet" use:enhance={sheetSubmit(() => (budgetOpen = false), m.money_budget_saved())}>
		<div class="sheet__head">
			<AgentShape agent="money" size={36} />
			<h2 class="md-headline-small-emphasized sheet__title">{m.money_monthly_budget()}</h2>
		</div>
		<TextField id="budget-category" name="category" label={m.common_category()} bind:value={budgetCategory} required maxlength={40} list="money-categories" />
		<TextField id="budget-limit" name="monthly_limit" label={m.money_limit()} type="number" inputmode="decimal" min="0.01" step="0.01" bind:value={budgetLimit} required />
		<div class="sheet__chips">
			{#each knownCategories as c (c)}
				<Chip variant="filter" type="button" selected={budgetCategory === c} onclick={() => (budgetCategory = c)}>{c}</Chip>
			{/each}
		</div>
		{#if form?.error}<p class="sheet__error" role="alert">{form.error}</p>{/if}
		<div class="sheet__actions">
			<Button type="button" variant="text" onclick={() => (budgetOpen = false)}>{m.common_cancel()}</Button>
			<Button type="submit" variant="filled" size="m" disabled={saving}>{m.money_save_budget()}</Button>
		</div>
	</form>
</BottomSheet>

<Snackbar bind:open={snackbar} message={snackbarMessage} />

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
		align-items: flex-end;
		justify-content: space-between;
		gap: 20px;
		flex-wrap: wrap;
	}
	.money__titles {
		flex: 1 1 360px;
	}
	.money__actions {
		display: flex;
		align-items: center;
		gap: 10px;
		flex-wrap: wrap;
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
	.budgets {
		list-style: none;
		margin: 0;
		padding: 0;
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(min(100%, 320px), 1fr));
		gap: 12px;
	}
	.budget {
		display: flex;
		flex-direction: column;
		gap: 10px;
		padding: 12px 8px 16px 16px;
		border-radius: var(--md-sys-shape-corner-large-increased);
		background: var(--md-sys-color-surface-container-low);
		transition: border-radius var(--nomi-motion-spatial-fast);
	}
	.budget:hover {
		border-radius: var(--md-sys-shape-corner-large);
	}
	.budget--over {
		background: color-mix(in srgb, var(--md-sys-color-error-container) 60%, var(--md-sys-color-surface-container-low));
	}
	.budget__top {
		display: flex;
		align-items: center;
		gap: 10px;
	}
	.budget__top .budget__name {
		flex: 1;
		min-width: 0;
	}
	.budget > :global(:last-child) {
		margin-right: 8px;
	}
	.budget__left {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.budget__row {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 8px;
		padding-right: 8px;
	}
	.budget__name {
		font-weight: 650;
		text-transform: capitalize;
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
	}
	.budget__figures {
		font-size: 1.25rem;
		font-weight: 650;
		font-variant-numeric: tabular-nums;
	}
	.budget__of {
		font-weight: 400;
		color: var(--md-sys-color-on-surface-variant);
	}
	.budget__flag {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		font-size: 0.8125rem;
		font-weight: 650;
		color: var(--md-sys-color-error);
	}
	.budget__flag--near {
		color: var(--md-sys-color-tertiary);
	}
	.sheet {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.sheet__head {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.sheet__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
	}
	.sheet__field {
		display: flex;
		flex-direction: column;
		gap: 6px;
	}
	.sheet__label {
		font-size: 0.8125rem;
		font-weight: 600;
		color: var(--md-sys-color-on-surface-variant);
	}
	.sheet__date {
		height: 56px;
		padding: 0 16px;
		border: 1px solid var(--md-sys-color-outline);
		border-radius: var(--md-sys-shape-corner-small);
		background: transparent;
		color: var(--md-sys-color-on-surface);
		font: inherit;
		color-scheme: light dark;
	}
	.sheet__chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}
	.sheet__error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.sheet__actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
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
	.tx__amount {
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		text-align: right;
	}
</style>
