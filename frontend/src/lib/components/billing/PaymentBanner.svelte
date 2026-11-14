<script lang="ts">
	// Shown on Billing when the person comes back from checkout or the invoice: what became of
	// that payment, and what to do next.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { invalidateAll } from '$app/navigation';
	import Button from '$lib/components/m3/Button.svelte';
	import { formatDate } from '$lib/billing';
	import type { BillingPayment } from '$lib/types';

	let { payment }: { payment: BillingPayment } = $props();

	let refreshing = $state(false);
	async function refresh() {
		refreshing = true;
		try {
			await invalidateAll();
		} finally {
			refreshing = false;
		}
	}

	const reasons: Record<'expired' | 'failed' | 'cancelled', () => string> = {
		expired: () => m.billing_banner_expired(),
		failed: () => m.billing_banner_failed(),
		cancelled: () => m.billing_banner_cancelled(),
	};
</script>

<section class="banner banner--{payment.status}" role="status" aria-live="polite">
	{#if payment.status === 'paid'}
		<h2 class="banner__title">{m.billing_banner_paid_title()}</h2>
		<p class="banner__body">
			{payment.period_end
				? m.billing_banner_paid_until({ plan: payment.plan_name, date: formatDate(payment.period_end, getLocale()) })
				: m.billing_banner_paid_now({ plan: payment.plan_name })}
		</p>
	{:else if payment.status === 'pending'}
		<h2 class="banner__title">{m.billing_banner_pending_title()}</h2>
		<p class="banner__body">{m.billing_banner_pending_body()}</p>
		<div class="banner__actions">
			<Button variant="filled" size="s" loading={refreshing} onclick={refresh}>{m.billing_banner_refresh()}</Button>
			{#if payment.invoice_url}<Button variant="outlined" size="s" href={payment.invoice_url}>{m.billing_banner_invoice()}</Button>{/if}
		</div>
	{:else}
		<h2 class="banner__title">{m.billing_banner_failed_title()}</h2>
		<p class="banner__body">{reasons[payment.status]()}</p>
		<div class="banner__actions">
			<Button variant="filled" size="s" href="/billing/checkout/{payment.plan_id}">{m.billing_banner_try_again()}</Button>
		</div>
	{/if}
</section>

<style>
	.banner {
		display: flex;
		flex-direction: column;
		gap: 6px;
		padding: 16px 20px;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
	}
	.banner--paid {
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.banner--expired,
	.banner--failed,
	.banner--cancelled {
		background: var(--md-sys-color-error-container);
		color: var(--md-sys-color-on-error-container);
	}
	.banner__title {
		margin: 0;
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.banner__body {
		margin: 0;
	}
	.banner__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: 6px;
	}
</style>
