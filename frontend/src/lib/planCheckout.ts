// Moving to a plan from Billing: what each plan's card offers, and spend in rupiah.

import type { Plan, PlansForUser } from '$lib/types';

/** What a plan's card offers: nothing (theirs, or the default), paying for it, or its waitlist. */
export type PlanOffer = 'none' | 'checkout' | 'waitlist' | 'waitlisted';

export function planOffer(plan: Plan, plans: PlansForUser): PlanOffer {
	if (plan.id === plans.current_plan_id || plan.is_default || !plan.is_active) return 'none';
	if (plan.price_idr && plans.checkout_available) return 'checkout';
	return plans.waitlist.includes(plan.id) ? 'waitlisted' : 'waitlist';
}

/** Rupiah as Indonesians write it, whatever the app's language: "Rp 1.800". */
export function formatIdr(amount: number): string {
	return new Intl.NumberFormat('id-ID', { style: 'currency', currency: 'IDR', maximumFractionDigits: 0, minimumFractionDigits: 0 })
		.format(Math.round(amount))
		.replace(/ /g, ' ');
}

/** Dollars at `rate` rupiah each, rounded to whole rupiah. */
export function usdToIdr(usd: number, rate: number): number {
	return Math.round(usd * rate);
}
