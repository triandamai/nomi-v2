<script lang="ts">
	// Checkout: the plan, how long to pay for, an optional code, and what it comes to. The total
	// always comes from the backend's quote; Pay sends the person to the invoice.
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { goto } from '$app/navigation';
	import Button from '$lib/components/m3/Button.svelte';
	import TextField from '$lib/components/m3/TextField.svelte';
	import IconClose from '$lib/components/icons/IconClose.svelte';
	import { CURRENCY, buildSummaryLines, currentMonthlyPrice, formatMoney, perMonth, planLimitLines, promoPriceLive, type SummaryLineKind, termLabel } from '$lib/billing';
	import { formatTokens } from '$lib/usage';
	import type { BillingQuote } from '$lib/types';
	import type { PageData } from './$types';

	let { data }: { data: PageData } = $props();

	const plan = $derived(data.plan);
	const currency = CURRENCY;
	const money = (amount: number) => formatMoney(amount, currency, getLocale());

	let months = $state<number>(0);
	let codeInput = $state('');
	let appliedCode = $state<string | null>(null);
	let codeError = $state<string | null>(null);
	let quote = $state<BillingQuote | null>(null);
	let quoting = $state(false);
	let quoteError = $state<string | null>(null);
	let paying = $state(false);
	let payError = $state<string | null>(null);
	// The newest request wins when answers come back out of order.
	let quoteRun = 0;

	// The first term, and its price, are there from the first paint.
	$effect.pre(() => {
		if (months === 0 && data.terms.length > 0) {
			months = data.terms[0].months;
			quote = data.quotes[months] ?? null;
		}
	});

	const term = $derived(data.terms.find((t) => t.months === months) ?? null);
	const lines = $derived(quote ? buildSummaryLines(quote) : []);
	const monthly = $derived(currentMonthlyPrice(plan));
	const hasPromo = $derived(promoPriceLive(plan));
	const lineLabels: Record<SummaryLineKind, () => string> = {
		subtotal: () => m.billing_checkout_subtotal(),
		promo: () => m.billing_checkout_promo(),
		term: () => m.billing_checkout_term_discount(),
		coupon: () => m.billing_checkout_coupon(),
		referral: () => m.billing_checkout_referral(),
		total: () => m.billing_checkout_total(),
	};

	async function post(path: string, code: string | null): Promise<Response> {
		return fetch(path, {
			method: 'POST',
			headers: { 'content-type': 'application/json' },
			body: JSON.stringify({ plan_id: plan.id, months, ...(code ? { code } : {}) }),
		});
	}

	/** The quote for the chosen term, with `code` when there is one. A refused code is said and dropped. */
	async function refresh(code: string | null) {
		const run = ++quoteRun;
		quoteError = null;
		if (!code && data.quotes[months]) {
			quote = data.quotes[months];
			appliedCode = null;
			return;
		}
		quoting = true;
		try {
			const response = await post('/billing/quote', code);
			if (run !== quoteRun) return;
			if (response.ok) {
				quote = (await response.json()) as BillingQuote;
				appliedCode = code;
				return;
			}
			const message = (await response.text()).trim() || m.billing_checkout_quote_failed();
			if (code) {
				codeError = message;
				appliedCode = null;
				await refresh(null);
			} else {
				quoteError = message;
			}
		} catch {
			if (run === quoteRun) quoteError = m.billing_checkout_quote_failed();
		} finally {
			if (run === quoteRun) quoting = false;
		}
	}

	function choose(next: number) {
		if (next === months) return;
		months = next;
		codeError = null;
		void refresh(appliedCode);
	}

	function apply(event: SubmitEvent) {
		event.preventDefault();
		const code = codeInput.trim();
		if (!code) return;
		codeError = null;
		void refresh(code);
	}

	function removeCode() {
		codeInput = '';
		codeError = null;
		void refresh(null);
	}

	async function pay() {
		if (!quote || paying) return;
		paying = true;
		payError = null;
		try {
			const response = await post('/billing/pay', appliedCode);
			if (!response.ok) {
				payError = (await response.text()).trim() || m.billing_checkout_pay_failed();
				return;
			}
			const result = (await response.json()) as { payment_id: string; status: string; invoice_url: string | null };
			if (result.status === 'paid' || !result.invoice_url) {
				await goto(`/billing?payment=${encodeURIComponent(result.payment_id)}`);
			} else {
				// Out to the payment provider's page; it returns the person to Billing when they're done.
				window.location.assign(result.invoice_url);
				return;
			}
		} catch {
			payError = m.billing_checkout_pay_failed();
		}
		paying = false;
	}
</script>

<div class="checkout">
	<div class="checkout__inner">
		<a class="back" href="/billing">{m.billing_checkout_back()}</a>
		<header>
			<h1 class="md-headline-medium checkout__title">{m.billing_checkout_title({ plan: plan.name })}</h1>
		</header>

		<div class="checkout__grid">
			<div class="checkout__main">
				<section class="panel" aria-labelledby="plan-heading">
					<h2 id="plan-heading" class="panel__title">{plan.name}</h2>
					{#if plan.description}<p class="muted">{plan.description}</p>{/if}
					<p class="price">
						<span class="price__now">{m.plan_price_per_month({ price: money(monthly) })}</span>
						{#if hasPromo}<s class="price__was">{money(plan.price_idr ?? 0)}</s>{/if}
					</p>
					<p class="tokens">{m.plan_tokens_month({ tokens: formatTokens(plan.monthly_tokens) })}</p>
					{#if plan.features.length > 0}
						<ul class="ticks">
							{#each plan.features as feature (feature)}
								<li>
									<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12 5 5 9-10" /></svg>
									{feature}
								</li>
							{/each}
						</ul>
					{/if}
					<p class="limits">{planLimitLines(plan).join(' · ')}</p>
				</section>

				<fieldset class="panel terms">
					<legend class="panel__title">{m.billing_checkout_term()}</legend>
					<div class="terms__list">
						{#each data.terms as option (option.id)}
							{@const priced = data.quotes[option.months]}
							<label class="term" class:term--on={months === option.months}>
								<input type="radio" name="term" value={option.months} checked={months === option.months} onchange={() => choose(option.months)} />
								<span class="term__name">
									{termLabel(option.months)}
									{#if option.discount_percent > 0}<span class="term__save">{m.billing_term_save({ percent: option.discount_percent })}</span>{/if}
								</span>
								{#if priced}
									<span class="term__price">{m.billing_checkout_per_month({ price: money(perMonth(priced.amount, option.months)) })}</span>
									<span class="term__total">{m.billing_checkout_total_for({ total: money(priced.amount) })}</span>
								{/if}
							</label>
						{:else}
							<p class="muted">{m.billing_checkout_no_terms()}</p>
						{/each}
					</div>
				</fieldset>

				<section class="panel" aria-labelledby="code-heading">
					<h2 id="code-heading" class="panel__title">{m.billing_checkout_code()}</h2>
					{#if appliedCode}
						<p class="chip">
							<span class="chip__text">{m.billing_checkout_code_applied({ code: appliedCode })}</span>
							<button type="button" class="chip__remove" onclick={removeCode} aria-label={m.billing_checkout_code_remove({ code: appliedCode })}><IconClose size={16} /></button>
						</p>
					{:else}
						<form class="code" onsubmit={apply}>
							<TextField label={m.billing_checkout_code()} bind:value={codeInput} autocomplete="off" autocapitalize="characters" error={codeError !== null} />
							<Button type="submit" variant="tonal" disabled={quoting || !codeInput.trim()}>{m.billing_checkout_apply()}</Button>
						</form>
					{/if}
					{#if codeError}<p class="error" role="alert">{codeError}</p>{/if}
				</section>
			</div>

			<aside class="panel summary" aria-labelledby="summary-heading">
				<h2 id="summary-heading" class="panel__title">{m.billing_checkout_summary()}</h2>
				{#if quote}
					<dl class="lines" aria-busy={quoting}>
						{#each lines as line (line.kind)}
							<div class="line line--{line.kind}">
								<dt>{lineLabels[line.kind]()}{#if line.code} <span class="line__code">{line.code}</span>{/if}</dt>
								<dd>{money(line.amount)}</dd>
							</div>
						{/each}
					</dl>
					{#if term}<p class="muted">{months === 1 ? m.billing_checkout_renews_monthly() : m.billing_checkout_renews({ months })}</p>{/if}
				{:else if quoteError}
					<p class="error" role="alert">{quoteError}</p>
				{/if}
				{#if payError}<p class="error" role="alert">{payError}</p>{/if}
				<Button variant="gradient" size="l" class="pay" loading={paying} disabled={!quote || quoting || paying} onclick={pay}>{m.billing_checkout_pay()}</Button>
			</aside>
		</div>
	</div>
</div>

<style>
	.checkout {
		height: 100%;
		overflow-y: auto;
		padding: 20px clamp(14px, 4vw, 44px) 44px;
		box-sizing: border-box;
	}
	.checkout__inner {
		max-width: 1020px;
		margin: 0 auto;
		display: flex;
		flex-direction: column;
		gap: 16px;
	}
	.back {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		min-height: 44px;
		color: var(--md-sys-color-primary);
		font-weight: 600;
		text-decoration: none;
	}
	.back:hover {
		text-decoration: underline;
	}
	.back:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.checkout__title {
		margin: 0;
		color: var(--md-sys-color-on-surface);
		overflow-wrap: anywhere;
	}
	.checkout__grid {
		display: grid;
		grid-template-columns: minmax(0, 1fr);
		gap: 14px;
		align-items: start;
	}
	@media (min-width: 900px) {
		.checkout__grid {
			grid-template-columns: minmax(0, 1.5fr) minmax(0, 1fr);
		}
		.summary {
			position: sticky;
			top: 0;
		}
	}
	.checkout__main {
		display: flex;
		flex-direction: column;
		gap: 14px;
		min-width: 0;
	}
	.panel {
		display: flex;
		flex-direction: column;
		gap: 10px;
		min-width: 0;
		margin: 0;
		padding: 18px 20px;
		box-sizing: border-box;
		border: none;
		border-radius: var(--md-sys-shape-corner-extra-large-increased);
		background: var(--md-sys-color-surface-container-lowest);
		color: var(--md-sys-color-on-surface);
	}
	.panel__title {
		margin: 0;
		padding: 0;
		font-size: 1.0625rem;
		font-weight: 700;
	}
	.muted {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.price {
		display: flex;
		flex-wrap: wrap;
		align-items: baseline;
		gap: 8px;
		margin: 0;
	}
	.price__now {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
	}
	.price__was {
		color: var(--md-sys-color-on-surface-variant);
	}
	.tokens {
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.ticks {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 0;
		padding: 0;
		list-style: none;
		font-size: 0.875rem;
	}
	.ticks li {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}
	.ticks svg {
		flex: none;
		margin-top: 2px;
		color: var(--md-sys-color-primary);
	}
	.limits {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.terms__list {
		display: grid;
		grid-template-columns: repeat(auto-fit, minmax(min(100%, 200px), 1fr));
		gap: 10px;
	}
	.term {
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 2px;
		min-height: 44px;
		padding: 12px 14px;
		border: 1px solid var(--md-sys-color-outline-variant);
		border-radius: var(--md-sys-shape-corner-large);
		background: var(--md-sys-color-surface-container-low);
		cursor: pointer;
	}
	.term:hover {
		background: var(--md-sys-color-surface-container);
	}
	.term--on {
		border-color: var(--md-sys-color-primary);
		outline: 1px solid var(--md-sys-color-primary);
		background: var(--md-sys-color-primary-container);
		color: var(--md-sys-color-on-primary-container);
	}
	.term:has(input:focus-visible) {
		outline: 2px solid var(--md-sys-color-primary);
		outline-offset: 2px;
	}
	.term input {
		position: absolute;
		opacity: 0;
		inset: 0;
		margin: 0;
		cursor: pointer;
	}
	.term__name {
		font-weight: 700;
	}
	.term__save {
		margin-left: 6px;
		font-size: 0.75rem;
		font-weight: 700;
		color: var(--md-sys-color-tertiary);
	}
	.term--on .term__save {
		color: inherit;
	}
	.term__price {
		font-size: 0.9375rem;
		font-variant-numeric: tabular-nums;
	}
	.term__total {
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
		font-variant-numeric: tabular-nums;
	}
	.term--on .term__total {
		color: inherit;
	}
	.code {
		display: flex;
		flex-wrap: wrap;
		align-items: flex-start;
		gap: 10px;
	}
	.code :global(.m3-text-field),
	.code > :global(*:first-child) {
		flex: 1 1 200px;
		min-width: 0;
	}
	.chip {
		display: inline-flex;
		align-items: center;
		align-self: flex-start;
		gap: 4px;
		min-height: 44px;
		margin: 0;
		padding: 0 6px 0 14px;
		border-radius: var(--md-sys-shape-corner-full);
		background: var(--md-sys-color-secondary-container);
		color: var(--md-sys-color-on-secondary-container);
		font-weight: 600;
		max-width: 100%;
	}
	.chip__text {
		overflow-wrap: anywhere;
	}
	.chip__remove {
		display: inline-flex;
		align-items: center;
		justify-content: center;
		width: 44px;
		height: 44px;
		border: none;
		border-radius: 50%;
		background: none;
		color: inherit;
		cursor: pointer;
	}
	.chip__remove:hover {
		background: color-mix(in srgb, currentColor 12%, transparent);
	}
	.chip__remove:focus-visible {
		outline: 2px solid var(--md-sys-color-primary);
	}
	.error {
		margin: 0;
		color: var(--md-sys-color-error);
		overflow-wrap: anywhere;
	}
	.lines {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin: 0;
	}
	.line {
		display: flex;
		justify-content: space-between;
		gap: 12px;
	}
	.line dt {
		color: var(--md-sys-color-on-surface-variant);
		overflow-wrap: anywhere;
	}
	.line dd {
		margin: 0;
		font-variant-numeric: tabular-nums;
		white-space: nowrap;
	}
	.line__code {
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
	}
	.line--total {
		padding-top: 10px;
		border-top: 1px solid var(--md-sys-color-outline-variant);
		font-weight: 700;
		font-size: 1.125rem;
	}
	.line--total dt {
		color: var(--md-sys-color-on-surface);
	}
	.summary :global(.pay) {
		width: 100%;
		min-height: 48px;
	}
</style>
