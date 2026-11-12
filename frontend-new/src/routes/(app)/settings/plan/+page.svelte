<script lang="ts">
	import { ArrowDown, KeyRound } from '@lucide/svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { clock } from '$lib/clock';
	import { dayKey } from '$lib/chat/dates';
	import { formatTokens } from '$lib/adapters/shell';
	import { dayMonth } from '$lib/settings/profile';
	import { allowance, dayBars, daysIn, fitNote, modelSpend, monthName, shiftMonth, usageStats, usd } from '$lib/settings/plan';
	import Button from '$lib/ds/actions/Button.svelte';
	import Segmented from '$lib/ds/inputs/Segmented.svelte';
	import StatStrip from '$lib/ds/profile/StatStrip.svelte';
	import StatPanel from '$lib/ds/profile/StatPanel.svelte';
	import UsageBars from '$lib/ds/profile/UsageBars.svelte';
	import PlanSummary from '$lib/ds/settings/PlanSummary.svelte';
	import MetaList from '$lib/ds/settings/MetaList.svelte';
	import MonthSwitch from '$lib/ds/settings/MonthSwitch.svelte';
	import DayBars, { type DayBar } from '$lib/ds/settings/DayBars.svelte';
	import PlanOption from '$lib/ds/settings/PlanOption.svelte';
	import FitNote from '$lib/ds/settings/FitNote.svelte';
	import SettingRow from '$lib/ds/settings/SettingRow.svelte';
	import SettingNote from '$lib/ds/settings/SettingNote.svelte';
	import PageToc from '$lib/ds/settings/PageToc.svelte';
	import SettingsTip from '$lib/ds/settings/SettingsTip.svelte';

	// Settings › Plan & usage (App / Settings / Plan & usage C0og49): the plan the person is on and
	// the billing facts beside it, a month of usage (its figures, a bar a day, by model), the plans
	// to compare with Nomi's word on the fit, and their own keys; On this page beside them.
	// Left out (AGENTS.md §9, BACKEND_TODO.md): joining a plan or its waitlist (there's no checkout),
	// the payment method, prices in rupiah for spend (the backend keeps dollars), a key's balance and
	// "Use Nomi allowance", and "Add a key" until Models & providers is in v2.
	let { data } = $props();

	const language = getLocale();
	const time = clock();
	const usage = $derived(data.usage);
	const today = $derived(dayKey(time.now, usage.timezone));
	const month = $derived(monthName(usage.month, language));
	const thisMonth = $derived(usage.month === usage.current_month);
	const allowed = $derived(allowance(usage, today));
	const plans = $derived((data.plans?.plans ?? []).filter((plan) => plan.is_active).sort((a, b) => a.sort_order - b.sort_order));
	const current = $derived(plans.find((plan) => plan.id === data.plans?.current_plan_id));
	const monthEnd = $derived(`${usage.month}-${String(daysIn(usage.month)).padStart(2, '0')}`);
	const weekdayDate = (date: string) =>
		`${new Intl.DateTimeFormat(language === 'id' ? 'id-ID' : 'en-US', { weekday: 'short', timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`))} ${dayMonth(new Date(`${date}T00:00:00Z`), language)}`;

	let unit = $state<'tokens' | 'spend'>('tokens');
	const chart = $derived(dayBars(usage, today, unit));
	const amount = (value: number) => (unit === 'tokens' ? formatTokens(value) : usd(value));
	const top = $derived(chart.peak?.value ?? 0);
	const describe = (bar: DayBar) => {
		const date = dayMonth(new Date(`${usage.month}-${String(bar.day).padStart(2, '0')}T00:00:00Z`), language);
		return unit === 'tokens' ? m.plan_day_tokens({ date, value: formatTokens(bar.value) }) : m.plan_day_spend({ date, value: usd(bar.value) });
	};

	const groups = $derived([
		{ href: '#plan', label: m.plan_yours() },
		{ href: '#usage', label: thisMonth ? m.plan_usage() : m.plan_usage_in({ month }) },
		...(plans.length ? [{ href: '#plans', label: m.plan_compare() }] : []),
		{ href: '#keys', label: m.plan_keys() },
	]);
	let scroller: HTMLElement | undefined = $state();
	let active = $state('#plan');
	function follow() {
		if (!scroller) return;
		const line = scroller.getBoundingClientRect().top + 96;
		const passed = groups.filter((group) => (document.querySelector(group.href)?.getBoundingClientRect().top ?? Infinity) <= line);
		active = passed.at(-1)?.href ?? groups[0].href;
	}
</script>

<svelte:head><title>{m.settings_plan()} · {m.app_name()}</title></svelte:head>

<div bind:this={scroller} onscroll={follow} class="h-full overflow-y-auto">
	<div class="flex gap-14 px-4 pt-8 pb-12 lg:px-12">
		<div class="flex max-w-220 min-w-0 flex-1 flex-col gap-8">
			<header class="flex flex-col gap-1.5">
				<h1 class="text-display font-medium tracking-display text-text">{m.settings_plan()}</h1>
				<p class="text-body text-text-2">{m.plan_intro()}</p>
			</header>

			<div id="plan" class="scroll-mt-6">
				<PlanSummary
					label={m.plan_yours()}
					tag={m.plan_tag({ name: usage.plan.name })}
					plan={usage.plan.name}
					line={usage.plan.custom_quota || !current
						? m.plan_line_custom({ allowance: formatTokens(usage.plan.monthly_tokens) })
						: /\d/.test(current.price_label)
							? m.plan_line({ price: current.price_label, allowance: formatTokens(usage.plan.monthly_tokens) })
							: m.plan_line_plain({ price: current.price_label, allowance: formatTokens(usage.plan.monthly_tokens) })}
					allowance={{ label: m.plan_allowance(), value: m.plan_allowance_value({ used: formatTokens(usage.tokens_used), allowance: formatTokens(usage.plan.monthly_tokens) }) }}
					used={allowed.used}
					projected={thisMonth ? allowed.projectedShare : undefined}
					legend={{
						used: m.plan_used({ percent: Math.round(allowed.used * 100) }),
						projected: thisMonth ? m.plan_projected({ tokens: formatTokens(allowed.projected), date: dayMonth(new Date(`${monthEnd}T00:00:00Z`), language) }) : undefined,
						resets: thisMonth
							? allowed.daysLeft === 1
								? m.plan_resets_one({ date: weekdayDate(allowed.resets) })
								: m.plan_resets({ date: weekdayDate(allowed.resets), days: allowed.daysLeft })
							: undefined,
					}}
				>
					{#snippet actions()}
						{#if plans.length}<Button label={m.plan_compare()} variant="secondary" href="#plans" iconEnd={ArrowDown} />{/if}
					{/snippet}
					{#snippet aside()}
						<MetaList
							label={m.plan_billing()}
							rows={[
								{ label: m.plan_resets_on(), value: weekdayDate(`${shiftMonth(usage.current_month, 1)}-01`) },
								{ label: m.plan_period(), value: `1–${dayMonth(new Date(`${monthEnd}T00:00:00Z`), language)}` },
								...(data.joinedAt ? [{ label: m.plan_member_since(), value: dayMonth(new Date(data.joinedAt), language, true) }] : []),
							]}
						/>
					{/snippet}
				</PlanSummary>
			</div>

			<section id="usage" aria-labelledby="usage-title" class="flex scroll-mt-6 flex-col gap-3">
				<header class="flex flex-wrap items-center justify-between gap-3">
					<div class="flex min-w-0 flex-col gap-0.5">
						<h2 id="usage-title" class="text-body font-medium text-text">{thisMonth ? m.plan_usage() : m.plan_usage_in({ month })}</h2>
						<p class="text-caption text-text-3">{m.plan_usage_body()}</p>
					</div>
					<MonthSwitch
						{month}
						previous={{ href: `?month=${shiftMonth(usage.month, -1)}`, label: monthName(shiftMonth(usage.month, -1), language) }}
						next={thisMonth ? undefined : { href: `?month=${shiftMonth(usage.month, 1)}`, label: monthName(shiftMonth(usage.month, 1), language) }}
						nextLabel={m.plan_no_later()}
					/>
				</header>
				<StatStrip small tiles={usageStats(usage)} />
				<div class="flex flex-col gap-4 lg:flex-row lg:items-start">
					<section aria-labelledby="days-title" class="flex min-w-0 flex-1 flex-col gap-4 rounded-md p-5 outline -outline-offset-1 outline-border">
						<header class="flex flex-wrap items-center gap-2.5">
							<h3 id="days-title" class="text-ui font-medium text-text">{m.plan_day_by_day()}</h3>
							<span class="min-w-0 flex-1 truncate font-mono text-label text-text-3">
								{chart.peak ? m.plan_peak({ value: amount(chart.peak.value), date: weekdayDate(chart.peak.date) }) : m.plan_no_peak()}
							</span>
							<Segmented
								label={m.plan_unit()}
								options={[
									{ value: 'tokens', label: m.plan_unit_tokens() },
									{ value: 'spend', label: m.plan_unit_spend() },
								]}
								value={unit}
								onchange={(value) => (unit = value as 'tokens' | 'spend')}
							/>
						</header>
						<DayBars label={m.plan_chart_label({ month })} bars={chart.bars} scale={[amount(top), amount(top / 2), '0']} today={m.plan_today()} {describe} />
					</section>
					<div class="lg:w-75 lg:shrink-0">
						<StatPanel title={m.plan_by_model()}>
							<UsageBars bars={modelSpend(usage)} empty={m.plan_by_model_empty()} />
						</StatPanel>
					</div>
				</div>
			</section>

			{#if plans.length}
				<section id="plans" aria-labelledby="plans-title" class="flex scroll-mt-6 flex-col gap-3">
					<header class="flex flex-col gap-0.5">
						<h2 id="plans-title" class="text-body font-medium text-text">{m.plan_compare()}</h2>
						<p class="text-caption text-text-3">{m.plan_compare_body()}</p>
					</header>
					<div class="grid gap-4 md:grid-cols-2">
						{#each plans as plan (plan.id)}
							{@const theirs = plan.id === data.plans?.current_plan_id}
							<PlanOption
								name={plan.name}
								tag={theirs ? { label: m.plan_current(), kind: 'go' } : plan.promo_label ? { label: plan.promo_label, kind: 'neutral' } : undefined}
								price={plan.promo_price_label ?? plan.price_label}
								per={m.plan_per_month()}
								description={plan.description}
								features={plan.features}
								raised={!theirs && current !== undefined && plan.monthly_tokens > current.monthly_tokens}
							/>
						{/each}
					</div>
					{#if thisMonth}<FitNote text={fitNote(allowed.projected, { name: usage.plan.name, monthly_tokens: usage.plan.monthly_tokens }, plans)} />{/if}
				</section>
			{/if}

			<section id="keys" aria-labelledby="keys-title" class="flex scroll-mt-6 flex-col gap-3">
				<header class="flex flex-col gap-0.5">
					<h2 id="keys-title" class="text-body font-medium text-text">{m.plan_keys()}</h2>
					<p class="text-caption text-text-3">{m.plan_keys_body()}</p>
				</header>
				<div class="flex flex-col border-t border-border">
					{#if data.keys === null}
						<SettingNote text={m.plan_keys_unloaded()} />
					{:else if data.keys.length === 0}
						<SettingNote text={m.plan_keys_none()} />
					{:else}
						{#each data.keys as key (key.id)}
							<SettingRow title={key.label} description={key.api_key_masked ?? key.base_url ?? key.provider_id} mono icon={KeyRound} />
						{/each}
					{/if}
				</div>
			</section>
		</div>

		<aside class="sticky top-0 hidden w-58 shrink-0 flex-col gap-5 self-start pt-24 xl:flex">
			<PageToc label={m.general_on_page()} links={groups} {active} />
			<SettingsTip kind="info" text={m.plan_tip()} />
		</aside>
	</div>
</div>
