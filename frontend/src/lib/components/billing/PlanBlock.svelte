<script lang="ts">
	// "Your plan" on Billing: the plan, how long it's paid for, auto-renew, and a renewal invoice
	// when one is waiting.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { enhance } from '$app/forms';
	import Button from '$lib/components/m3/Button.svelte';
	import Switch from '$lib/components/m3/Switch.svelte';
	import { formatDate, formatMoney } from '$lib/billing';
	import type { BillingSubscription } from '$lib/types';

	let { subscription, error = null }: { subscription: BillingSubscription; error?: string | null } = $props();

	let autoRenew = $state(false);
	$effect(() => {
		autoRenew = subscription.auto_renew;
	});
	let formEl: HTMLFormElement | undefined = $state();
	const renewal = $derived(subscription.pending_renewal);
</script>

<section class="panel" aria-labelledby="your-plan-heading">
	<h2 id="your-plan-heading" class="panel__title">{m.billing_your_plan()}</h2>
	<p class="name">{subscription.plan.name}</p>
	<p class="muted">
		{subscription.paid_until ? m.billing_plan_until({ date: formatDate(subscription.paid_until, getLocale()) }) : m.billing_plan_no_end()}
		{#if subscription.bonus_days > 0} · {m.billing_plan_bonus({ days: subscription.bonus_days })}{/if}
	</p>
	{#if subscription.lapsed}
		<p class="notice" role="status">{m.billing_plan_lapsed()}</p>
	{/if}
	{#if subscription.paid_until}
		<form bind:this={formEl} method="POST" action="?/autoRenew" use:enhance class="auto">
			<Switch name="auto_renew" bind:checked={autoRenew} aria-label={m.billing_auto_renew()} onchange={() => formEl?.requestSubmit()} />
			<span class="auto__text">
				<span class="auto__label">{m.billing_auto_renew()}</span>
				<span class="muted">{m.billing_auto_renew_hint()}</span>
			</span>
		</form>
	{/if}
	{#if renewal && renewal.status === 'pending'}
		<div class="renewal">
			<span>{m.billing_renewal_due({ amount: formatMoney(renewal.amount, renewal.currency, getLocale()) })}</span>
			{#if renewal.invoice_url}<Button variant="filled" size="s" href={renewal.invoice_url}>{m.billing_pay_now()}</Button>{/if}
		</div>
	{/if}
	{#if error}<p class="error" role="alert">{error}</p>{/if}
</section>

<style>
	.panel {
		display: flex;
		flex-direction: column;
		gap: 8px;
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
	.name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 800;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.notice {
		margin: 0;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-surface-container-low);
		font-size: 0.875rem;
	}
	.auto {
		display: flex;
		align-items: flex-start;
		gap: 12px;
		margin: 0;
	}
	.auto__text {
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-width: 0;
	}
	.auto__label {
		font-weight: 600;
	}
	.renewal {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		justify-content: space-between;
		gap: 8px 12px;
		padding: 10px 12px;
		border-radius: var(--md-sys-shape-corner-medium);
		background: var(--md-sys-color-tertiary-container);
		color: var(--md-sys-color-on-tertiary-container);
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
	}
</style>
