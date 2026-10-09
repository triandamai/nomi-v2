<script lang="ts">
	import { m } from '$lib/paraglide/messages';
	import { getLocale } from '$lib/paraglide/runtime';
	import { GRADIENT_STOPS, GRADIENT_TONES, TONE_ACCENT, type GradientTone } from '$lib/components/m3/shapes';
	import { formatTokens } from '$lib/usage';
	import type { Plan } from '$lib/types';
	import type { Snippet } from 'svelte';

	// One plan as a card: a band in the plan's gradient, its price (and promo, while it runs),
	// allowance and features. Billing, the plans sheet and Admin → Plans all show plans this way.
	let {
		plan,
		current = false,
		actions,
	}: {
		plan: Plan;
		/** The person is on it. */
		current?: boolean;
		actions?: Snippet;
	} = $props();

	const tone = $derived<GradientTone>((GRADIENT_TONES as readonly string[]).includes(plan.card_tone) ? (plan.card_tone as GradientTone) : 'glow');
	const band = $derived(`linear-gradient(110deg, ${GRADIENT_STOPS[tone].join(', ')})`);
	const promoLive = $derived(Boolean(plan.promo_label || plan.promo_price_label) && (!plan.promo_ends_at || new Date(plan.promo_ends_at) > new Date()));
	const promoEnds = $derived(
		plan.promo_ends_at ? new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'long' }).format(new Date(plan.promo_ends_at)) : null,
	);
</script>

<article class="plan" class:plan--current={current} style:--plan-accent={TONE_ACCENT[tone]}>
	<div class="plan__band" style:background={band}>
		{#if promoLive && plan.promo_label}<span class="plan__promo">{plan.promo_label}</span>{/if}
		{#if current}<span class="plan__current">{m.billing_current()}</span>{/if}
	</div>
	<div class="plan__body">
		<h3 class="plan__name">{plan.name}</h3>
		{#if plan.description}<p class="plan__description">{plan.description}</p>{/if}
		<p class="plan__price">
			{#if promoLive && plan.promo_price_label}
				<span class="plan__price-now">{plan.promo_price_label}</span>
				{#if plan.price_label}<s class="plan__price-was">{plan.price_label}</s>{/if}
			{:else}
				<span class="plan__price-now">{plan.price_label || '—'}</span>
			{/if}
		</p>
		{#if promoLive && promoEnds}<p class="plan__promo-ends">{m.plan_promo_until({ date: promoEnds })}</p>{/if}
		<p class="plan__tokens">{m.plan_tokens_month({ tokens: formatTokens(plan.monthly_tokens) })}</p>
		{#if plan.features.length > 0}
			<ul class="plan__features">
				{#each plan.features as feature (feature)}
					<li>
						<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="m5 12 5 5 9-10" /></svg>
						{feature}
					</li>
				{/each}
			</ul>
		{/if}
		{#if actions}<div class="plan__actions">{@render actions()}</div>{/if}
	</div>
</article>

<style>
	.plan {
		display: flex;
		flex-direction: column;
		overflow: hidden;
		border-radius: var(--md-sys-shape-corner-extra-large);
		background: var(--md-sys-color-surface-container-low);
		color: var(--md-sys-color-on-surface);
	}
	.plan--current {
		outline: 2px solid var(--plan-accent);
		outline-offset: -2px;
	}
	.plan__band {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: 8px;
		min-height: 56px;
		padding: 12px 14px;
	}
	.plan__promo,
	.plan__current {
		padding: 4px 10px;
		border-radius: var(--md-sys-shape-corner-full);
		font-size: 0.75rem;
		font-weight: 700;
	}
	.plan__promo {
		background: rgba(255, 255, 255, 0.85);
		color: #3a1d00;
	}
	.plan__current {
		margin-left: auto;
		background: rgba(15, 26, 20, 0.82);
		color: #fff;
	}
	.plan__body {
		display: flex;
		flex: 1;
		flex-direction: column;
		gap: 8px;
		padding: 16px 20px 20px;
	}
	.plan__name {
		margin: 0;
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.375rem;
		font-weight: 800;
		letter-spacing: -0.02em;
	}
	.plan__description {
		margin: 0;
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.plan__price {
		display: flex;
		align-items: baseline;
		flex-wrap: wrap;
		gap: 8px;
		margin: 4px 0 0;
	}
	.plan__price-now {
		font-family: var(--md-ref-typeface-brand);
		font-size: 1.25rem;
		font-weight: 700;
		color: var(--plan-accent);
	}
	.plan__price-was {
		color: var(--md-sys-color-on-surface-variant);
		font-size: 0.875rem;
	}
	.plan__promo-ends {
		margin: 0;
		font-size: 0.8125rem;
		color: var(--md-sys-color-on-surface-variant);
	}
	.plan__tokens {
		margin: 0;
		font-family: var(--md-ref-typeface-mono);
		font-size: 0.8125rem;
		font-weight: 600;
	}
	.plan__features {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin: 4px 0 0;
		padding: 0;
		list-style: none;
		font-size: 0.875rem;
	}
	.plan__features li {
		display: flex;
		align-items: flex-start;
		gap: 8px;
	}
	.plan__features svg {
		flex: none;
		margin-top: 2px;
		color: var(--plan-accent);
	}
	.plan__actions {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		margin-top: auto;
		padding-top: 8px;
	}
</style>
