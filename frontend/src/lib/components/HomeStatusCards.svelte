<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import Button from '$lib/components/m3/Button.svelte';
	import OutList from '$lib/components/home/OutList.svelte';
	import PlanList from '$lib/components/home/PlanList.svelte';
	import TodayList from '$lib/components/home/TodayList.svelte';
	import type { HomeSummary } from '$lib/types';

	// Home's three status cards (design: Home artboard): what moved while the user was away,
	// what's due today, and plans still in progress. Each shows up to four; the rest are on the
	// section's own page.

	let { summary }: { summary: HomeSummary } = $props();
</script>

{#snippet more(total: number, shown: number, href: string, what: string)}
	{#if total > shown}
		<Button variant="tonal" size="s" {href} class="card__more" aria-label={m.home_show_all({ count: total, what })}>{m.home_show_more({ count: total - shown })}</Button>
	{/if}
{/snippet}

<div class="cards">
	<section class="card" aria-labelledby="out-heading">
		<h2 id="out-heading" class="card__title">{m.home_out_title()}</h2>
		{#if summary.while_you_were_out.length === 0}
			<p class="card__empty">{m.home_out_empty()}</p>
		{:else}
			<OutList items={summary.while_you_were_out} />
			{@render more(summary.while_you_were_out_total, summary.while_you_were_out.length, '/home/updates', m.home_what_updates())}
		{/if}
	</section>

	<section class="card" aria-labelledby="today-heading">
		<div class="card__head">
			<h2 id="today-heading" class="card__title">{m.home_today_title()}</h2>
			<a class="card__link" href="/reminders">{m.nav_reminders()}</a>
		</div>
		{#if summary.today.length === 0}
			<p class="card__empty">{m.home_today_empty()}</p>
		{:else}
			<TodayList items={summary.today} timezone={summary.timezone} />
			{@render more(summary.today_total, summary.today.length, '/home/today', m.home_what_today())}
		{/if}
	</section>

	<section class="card" aria-labelledby="plans-heading">
		<h2 id="plans-heading" class="card__title">{m.home_plans_title()}</h2>
		{#if summary.plans.length === 0}
			<p class="card__empty">{m.home_plans_empty()}</p>
		{:else}
			<PlanList items={summary.plans} />
			{@render more(summary.plans_total, summary.plans.length, '/home/plans', m.home_what_plans())}
		{/if}
	</section>
</div>

<style>

	.cards {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 300px), 1fr));
		gap: 16px;
		align-items: start;
	}
	.card {
		display: flex;
		flex-direction: column;
		gap: 12px;
		padding: 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
		min-width: 0;
	}
	.card__head {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
	}
	.card__title {
		margin: 0;
		font-size: 1.125rem;
		font-weight: 700;
	}
	.card__link {
		color: var(--md-sys-color-primary);
		font-weight: 600;
		font-size: 0.875rem;
		text-decoration: none;
		padding: 4px 2px;
	}
	.card__empty {
		margin: 0;
		font-size: 0.9375rem;
		line-height: 1.5;
		color: var(--md-sys-color-on-surface-variant);
	}
	.card :global(.card__more) {
		align-self: flex-start;
	}
</style>
