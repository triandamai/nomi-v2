<script lang="ts">
	// The project's tokens and spend, a month at a time: everything the crew used in its chat and
	// the chats moved into it, by model, and by person for the project's admins. Calls made before
	// calls carried their chat count toward no project, so the sheet says since when it counts
	// rather than show a zero that looks like nothing happened.
	import { onMount } from 'svelte';
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import Button from '$lib/components/m3/Button.svelte';
	import IconButton from '$lib/components/m3/IconButton.svelte';
	import IconChevronLeft from '$lib/components/icons/IconChevronLeft.svelte';
	import IconChevronRight from '$lib/components/icons/IconChevronRight.svelte';
	import Money from '$lib/components/Money.svelte';
	import { monthLabel, shiftMonth } from '$lib/money';
	import { formatIdr, usdToIdr } from '$lib/planCheckout';
	import { projectApi } from '$lib/projectApi';
	import { formatTokens, formatTokensFull, formatUsd } from '$lib/usage';
	import type { ProjectUsage } from '$lib/types';

	let { projectId }: { projectId: string } = $props();

	let usage = $state<ProjectUsage | null>(null);
	let failed = $state(false);
	let loading = $state(true);
	let asked = $state<string | null>(null);

	async function load(month: string | null) {
		asked = month;
		loading = true;
		failed = false;
		try {
			const next = await projectApi<ProjectUsage>(projectId, month ? `usage?month=${month}` : 'usage');
			// A slow answer for a month the person already left is dropped.
			if (asked !== month) return;
			usage = next;
		} catch {
			if (asked === month) failed = true;
		} finally {
			if (asked === month) loading = false;
		}
	}

	onMount(() => {
		void load(null);
	});

	const isCurrent = $derived(usage ? usage.month === usage.current_month : true);
	const spend = (usd: number, rate: number) => m.billing_spend_idr({ idr: formatIdr(usdToIdr(usd, rate)), usd: formatUsd(usd) });
	const sinceLabel = $derived(
		usage?.counted_since ? new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(usage.counted_since)) : null,
	);
</script>

<div class="usage">
	<p class="muted">{m.pj_usage_scope()}</p>

	{#if usage}
		<nav class="months" aria-label={m.money_month()}>
			<IconButton variant="filled-tonal" onclick={() => load(shiftMonth(asked ?? usage!.month, -1))} aria-label={m.money_prev_month()}><IconChevronLeft /></IconButton>
			<span class="months__current" aria-live="polite">{monthLabel(usage.month)}</span>
			<IconButton variant="filled-tonal" disabled={isCurrent} onclick={() => load(shiftMonth(asked ?? usage!.month, 1))} aria-label={m.money_next_month()}><IconChevronRight /></IconButton>
		</nav>
	{/if}

	{#if failed}
		<div class="state" role="alert">
			<p class="error">{m.pj_usage_load_failed()}</p>
			<Button variant="tonal" size="s" onclick={() => load(asked)}>{m.pj_usage_retry()}</Button>
		</div>
	{:else if !usage}
		<p class="muted" role="status">{m.pj_usage_loading()}</p>
	{:else}
		<div class="body" class:body--loading={loading} aria-busy={loading}>
			<dl class="tiles">
				<div class="tile">
					<dt>{m.pj_usage_spend()}</dt>
					<dd><Money value={spend(usage.spend_usd, usage.usd_to_idr)} /></dd>
				</div>
				<div class="tile">
					<dt>{m.pj_usage_tokens()}</dt>
					<dd title={formatTokensFull(usage.tokens)}>{formatTokens(usage.tokens)}</dd>
				</div>
				<div class="tile">
					<dt>{m.pj_usage_calls()}</dt>
					<dd>{formatTokensFull(usage.calls)}</dd>
				</div>
			</dl>
			{#if usage.own_key_tokens > 0}
				<p class="meta">{m.pj_usage_own_key({ tokens: formatTokens(usage.own_key_tokens) })}</p>
			{/if}

			{#if usage.calls === 0}
				<p class="empty">{m.pj_usage_empty({ month: monthLabel(usage.month) })}</p>
			{:else}
				<h3 class="section-title">{m.pj_usage_by_model()}</h3>
				<div class="table-wrap">
					<table>
						<thead><tr><th scope="col">{m.pj_usage_model()}</th><th scope="col" class="num">{m.pj_usage_tokens()}</th><th scope="col" class="num">{m.pj_usage_spend()}</th></tr></thead>
						<tbody>
							{#each usage.by_model as row (`${row.admin_model_id}-${row.provider}-${row.model_id}-${row.source}`)}
								<tr>
									<td>{row.label}{#if row.source === 'own_key'} <span class="tag">{m.pj_usage_own_key_tag()}</span>{/if}</td>
									<td class="num" title={formatTokensFull(row.tokens)}>{formatTokens(row.tokens)}</td>
									<td class="num">{formatIdr(usdToIdr(row.spend_usd, usage.usd_to_idr))}</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>

				{#if usage.shows_people && usage.by_person.length > 0}
					<h3 class="section-title">{m.pj_usage_by_person()}</h3>
					<div class="table-wrap">
						<table>
							<thead><tr><th scope="col">{m.pj_usage_person()}</th><th scope="col" class="num">{m.pj_usage_tokens()}</th><th scope="col" class="num">{m.pj_usage_spend()}</th></tr></thead>
							<tbody>
								{#each usage.by_person as row (row.user_id)}
									<tr>
										<td>{row.name ?? m.pj_usage_unknown_person()}</td>
										<td class="num" title={formatTokensFull(row.tokens)}>{formatTokens(row.tokens)}</td>
										<td class="num">{formatIdr(usdToIdr(row.spend_usd, usage.usd_to_idr))}</td>
									</tr>
								{/each}
							</tbody>
						</table>
					</div>
				{/if}
			{/if}

			<p class="caption">{sinceLabel ? m.pj_usage_counted_since({ date: sinceLabel }) : m.pj_usage_counting_starts()}</p>
		</div>
	{/if}
</div>

<style>
	.usage {
		display: flex;
		flex-direction: column;
		gap: 12px;
		max-width: 720px;
	}
	.muted,
	.meta,
	.caption,
	.empty {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.caption {
		margin-top: 16px;
		font-size: 0.75rem;
	}
	.empty {
		padding: 24px 18px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
		font-size: 0.875rem;
	}
	.state {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 12px;
	}
	.months {
		display: flex;
		align-items: center;
		gap: 12px;
	}
	.months__current {
		min-width: 9rem;
		text-align: center;
		color: var(--md-sys-color-on-surface);
		font-weight: 600;
	}
	.body--loading {
		opacity: 0.6;
	}
	.tiles {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
		gap: 8px;
		margin: 0 0 8px;
	}
	.tile {
		padding: 14px 16px;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	.tile dt {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.tile dd {
		margin: 4px 0 0;
		color: var(--md-sys-color-on-surface);
		font-size: 1.25rem;
		font-weight: 600;
		font-variant-numeric: tabular-nums;
	}
	.section-title {
		margin: 20px 0 8px;
		color: var(--md-sys-color-on-surface);
		font-size: 0.9375rem;
		font-weight: 700;
	}
	.table-wrap {
		overflow-x: auto;
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-lowest);
	}
	table {
		width: 100%;
		border-collapse: collapse;
		font-size: 0.875rem;
	}
	th,
	td {
		padding: 10px 16px;
		text-align: left;
		color: var(--md-sys-color-on-surface);
	}
	th {
		color: var(--md-sys-color-on-surface-variant);
		font-weight: 500;
	}
	tbody tr + tr td {
		border-top: 1px solid var(--md-sys-color-outline-variant);
	}
	.num {
		text-align: right;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.tag {
		margin-left: 6px;
		padding: 1px 8px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.75rem;
	}
</style>
