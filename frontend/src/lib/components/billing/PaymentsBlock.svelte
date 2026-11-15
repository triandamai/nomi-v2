<script lang="ts">
	// "Payments" on Billing: a plain list, newest first, with Pay and Cancel on the ones still waiting.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Pagination from '$lib/components/m3/Pagination.svelte';
	import PaymentStatus from '$lib/components/billing/PaymentStatus.svelte';
	import { formatDate, formatMoney, termLabel } from '$lib/billing';
	import { pageCount, pageSlice } from '$lib/pagination';
	import type { BillingPayment } from '$lib/types';

	let { payments, error = null }: { payments: BillingPayment[]; error?: string | null } = $props();

	/** Payments on one page of the list. */
	const PAGE_SIZE = 10;
	let page = $state(1);
	const newest = $derived(payments.toSorted((a, b) => b.created_at.localeCompare(a.created_at)));
	const pages = $derived(pageCount(newest.length, PAGE_SIZE));
	const shown = $derived(pageSlice(newest, Math.min(page, pages), PAGE_SIZE));
</script>

<section class="panel" aria-labelledby="payments-heading">
	<h2 id="payments-heading" class="panel__title">{m.billing_payments()}</h2>
	{#if error}<p class="error" role="alert">{error}</p>{/if}
	{#if payments.length === 0}
		<p class="muted">{m.billing_payments_none()}</p>
	{:else}
		<ul class="rows">
			{#each shown as payment (payment.id)}
				<li class="row">
					<div class="row__text">
						<span class="row__title">
							{payment.plan_name} · {termLabel(payment.term_months)}
							{#if payment.kind === 'renewal'}<span class="muted">· {m.billing_payment_renewal()}</span>{/if}
						</span>
						<span class="muted">{formatDate(payment.paid_at ?? payment.created_at, getLocale())}</span>
					</div>
					<span class="row__amount">{formatMoney(payment.amount, payment.currency, getLocale())}</span>
					<PaymentStatus status={payment.status} />
					{#if payment.status === 'pending'}
						<div class="row__actions">
							{#if payment.invoice_url}<Button variant="tonal" size="xs" href={payment.invoice_url}>{m.billing_payment_pay()}</Button>{/if}
							<form method="POST" action="?/cancel" use:enhance>
								<input type="hidden" name="id" value={payment.id} />
								<Button type="submit" variant="text" size="xs">{m.common_cancel()}</Button>
							</form>
						</div>
					{/if}
				</li>
			{/each}
		</ul>
		{#if pages > 1}
			<Pagination page={Math.min(page, pages)} {pages} onselect={(next) => (page = next)} />
		{/if}
	{/if}
</section>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		padding: 18px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.panel__title {
		margin: 0;
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.8125rem;
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.rows {
		display: flex;
		flex-direction: column;
		margin: 0;
		padding: 0;
		list-style: none;
	}
	.row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: 6px 12px;
		padding: 10px 0;
		border-bottom: 1px solid var(--md-sys-color-outline-variant);
	}
	.row:last-child {
		border-bottom: none;
	}
	.row__text {
		display: flex;
		flex: 1 1 180px;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.row__title {
		font-weight: 600;
		overflow-wrap: anywhere;
	}
	.row__amount {
		font-variant-numeric: tabular-nums;
		font-weight: 600;
	}
	.row__actions {
		display: flex;
		flex-basis: 100%;
		gap: 8px;
	}
</style>
