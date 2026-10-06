<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import OutList from '$lib/components/home/OutList.svelte';
	import PlanList from '$lib/components/home/PlanList.svelte';
	import TodayList from '$lib/components/home/TodayList.svelte';
	import type { OutItem, PlanItem, TodayItem } from '$lib/home';
	import { pageCount } from '$lib/pagination';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	// One of Home's cards in full: every item, a page at a time.
	const result = $derived(data.result);
	const pages = $derived(result ? pageCount(result.total, result.per_page) : 1);

	const EMPTY: Record<typeof data.section, string> = {
		updates: m.home_out_empty(),
		today: m.home_today_empty(),
		plans: m.home_plans_empty(),
	};
</script>

<div class="section-page">
	<div class="section-page__inner">
		<header class="section-page__head">
			<IconButton href="/" aria-label={m.home_back()}><IconChevronLeft /></IconButton>
			<div>
				<h1 class="md-display-small section-page__title">{data.title}</h1>
				<p class="md-body-large section-page__lede">
					{data.lede}
					{#if result && result.total > 0}<span class="nomi-meta section-page__count">{m.home_in_all({ count: result.total })}</span>{/if}
				</p>
			</div>
		</header>

		{#if !result}
			<p class="section-page__empty" role="alert">{m.home_load_failed()}</p>
		{:else if result.items.length === 0}
			<p class="section-page__empty">{result.total > 0 ? m.home_no_more() : EMPTY[data.section]}</p>
			{#if result.total > 0}<Pagination page={result.page} {pages} href={(p) => `?page=${p}`} />{/if}
		{:else}
			<section class="section-page__card">
				{#if data.section === 'updates'}
					<OutList items={result.items as OutItem[]} />
				{:else if data.section === 'today'}
					<TodayList items={result.items as TodayItem[]} timezone={result.timezone} />
				{:else}
					<PlanList items={result.items as PlanItem[]} />
				{/if}
			</section>
			<Pagination page={result.page} {pages} href={(p) => `?page=${p}`} label={m.home_pages({ title: data.title })} />
		{/if}
	</div>
</div>

<style>
	.section-page {
		height: 100%;
		overflow-y: auto;
		padding: 32px clamp(16px, 4vw, 56px) 56px;
		box-sizing: border-box;
	}
	.section-page__inner {
		display: flex;
		flex-direction: column;
		gap: 24px;
		max-width: 760px;
		margin: 0 auto;
	}
	.section-page__head {
		display: flex;
		align-items: flex-start;
		gap: 8px;
		margin-left: -8px;
	}
	.section-page__title {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface);
	}
	.section-page__lede {
		margin: 8px 0 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.section-page__count {
		display: block;
		margin-top: 8px;
	}
	.section-page__empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
	}
	.section-page__card {
		padding: 24px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
</style>
