// Helpers for paid plans: how amounts read, which plans are for sale, and the order summary a
// quote turns into. Pure, so they can be tested without a page.
import { formatStorage } from '$lib/storage';
import { m } from '$lib/paraglide/messages';
import type { BillingQuote, PaymentStatus, Plan, PlansForUser } from '$lib/types';

/** The currency every plan is sold in (a plan's price is `price_idr`). */
export const CURRENCY = 'IDR';

/** Currencies that are priced in whole units, whatever the platform's own locale data says. */
const WHOLE_UNIT_CURRENCIES = new Set(['IDR', 'JPY', 'KRW', 'VND']);

/** `1500000` as "Rp 1.500.000" (IDR has no decimals; other currencies show the ones they use). */
export function formatMoney(amount: number, currency: string, locale: string): string {
	try {
		const wholeUnits = WHOLE_UNIT_CURRENCIES.has(currency.toUpperCase()) ? { minimumFractionDigits: 0, maximumFractionDigits: 0 } : {};
		return new Intl.NumberFormat(locale, { style: 'currency', currency, currencyDisplay: 'narrowSymbol', ...wholeUnits }).format(amount);
	} catch {
		// An unknown currency code: the amount with the code beside it.
		return `${new Intl.NumberFormat(locale).format(amount)} ${currency}`;
	}
}

/** Whether the plan's promo price applies right now. */
export function promoPriceLive(plan: Pick<Plan, 'price_idr' | 'promo_price_idr' | 'promo_ends_at'>, now: Date = new Date()): boolean {
	if (typeof plan.promo_price_idr !== 'number' || plan.promo_price_idr >= (plan.price_idr ?? 0)) return false;
	return !plan.promo_ends_at || new Date(plan.promo_ends_at) > now;
}

/** The price a person pays per month today: the promo's while it runs, else the list price. */
export function currentMonthlyPrice(plan: Pick<Plan, 'price_idr' | 'promo_price_idr' | 'promo_ends_at'>, now: Date = new Date()): number {
	return promoPriceLive(plan, now) ? (plan.promo_price_idr as number) : (plan.price_idr ?? 0);
}

/** Plans someone can choose in the upgrade sheet: priced, shown, and not the one they're on. */
export function plansForSale(offer: Pick<PlansForUser, 'plans' | 'current_plan_id'>): Plan[] {
	return offer.plans.filter((plan) => plan.is_active && (plan.price_idr ?? 0) > 0 && plan.id !== offer.current_plan_id);
}

/** Whole units per month for a total over `months`. */
export function perMonth(total: number, months: number): number {
	return months > 0 ? Math.round(total / months) : total;
}

export type SummaryLineKind = 'subtotal' | 'promo' | 'term' | 'coupon' | 'referral' | 'total';

export interface SummaryLine {
	kind: SummaryLineKind;
	/** Positive for what is charged, negative for what comes off. */
	amount: number;
	/** The code behind a coupon or referral line. */
	code?: string;
}

/** The order summary a quote shows: the subtotal, every discount that isn't zero, the total. */
export function buildSummaryLines(quote: BillingQuote): SummaryLine[] {
	const lines: SummaryLine[] = [{ kind: 'subtotal', amount: quote.base_amount }];
	const discounts: [SummaryLineKind, number, string | undefined][] = [
		['promo', quote.promo_discount, undefined],
		['term', quote.term_discount, undefined],
		['coupon', quote.coupon_discount, quote.coupon_code ?? undefined],
		['referral', quote.referral_discount, quote.referral_code ?? undefined],
	];
	for (const [kind, amount, code] of discounts) {
		if (amount > 0) lines.push({ kind, amount: -amount, code });
	}
	lines.push({ kind: 'total', amount: quote.amount });
	return lines;
}

/** How full a limit is, 0..1; null when the limit is unlimited. */
export function meterShare(used: number, limit: number | null): number | null {
	if (limit === null) return null;
	if (limit <= 0) return used > 0 ? 1 : 0;
	return Math.min(1, Math.max(0, used / limit));
}

/** Every state a payment can be in, as the backend names them. */
export const PAYMENT_STATUSES: PaymentStatus[] = ['pending', 'paid', 'expired', 'failed', 'cancelled'];

const STATUS_LABELS: Record<PaymentStatus, () => string> = {
	pending: () => m.billing_status_pending(),
	paid: () => m.billing_status_paid(),
	expired: () => m.billing_status_expired(),
	failed: () => m.billing_status_failed(),
	cancelled: () => m.billing_status_cancelled(),
};

export function paymentStatusLabel(status: PaymentStatus): string {
	return STATUS_LABELS[status]();
}

/** What a plan allows, one short phrase each: the key limits shown on the checkout. */
export function planLimitLines(plan: Plan): string[] {
	const lines = [
		plan.max_projects === null ? m.plans_admin_sum_projects_unlimited() : m.plans_admin_sum_projects({ count: plan.max_projects }),
		plan.max_orgs === null ? m.plans_admin_sum_orgs_unlimited() : m.plans_admin_sum_orgs({ count: plan.max_orgs }),
		plan.max_org_members === null ? m.plans_admin_sum_members_unlimited() : m.plans_admin_sum_members({ count: plan.max_org_members }),
		plan.max_custom_crew === null ? m.plans_admin_sum_crew_unlimited() : m.plans_admin_sum_crew({ count: plan.max_custom_crew }),
		plan.max_storage_mb === null ? m.plans_admin_sum_storage_unlimited() : formatStorage(plan.max_storage_mb),
	];
	if (plan.custom_domain) lines.push(m.plans_admin_sum_custom_domain());
	lines.push(plan.all_models ? m.plans_admin_sum_models_all() : m.plans_admin_sum_models({ count: plan.model_ids.length }));
	return lines;
}

/** A day as the locale writes it: "15 Nov 2026". */
export function formatDate(iso: string, locale: string): string {
	return new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'short', year: 'numeric' }).format(new Date(iso));
}

/** "1 month", "6 months". */
export function termLabel(months: number): string {
	return months === 1 ? m.billing_term_one() : m.billing_term_months({ count: months });
}
