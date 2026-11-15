import { describe, expect, it } from 'vitest';
import { buildSummaryLines, currentMonthlyPrice, formatMoney, meterShare, perMonth, plansForSale, promoPriceLive } from './billing';
import type { BillingQuote, Plan } from './types';

const quote: BillingQuote = {
	plan_id: 'p1',
	term_months: 6,
	currency: 'IDR',
	monthly_price: 100_000,
	base_amount: 600_000,
	promo_discount: 0,
	term_discount: 60_000,
	coupon_discount: 0,
	referral_discount: 0,
	amount: 540_000,
	coupon_code: null,
	referral_code: null,
};

describe('formatMoney', () => {
	it('shows rupiah without decimals, grouped the way the locale does', () => {
		expect(formatMoney(1_500_000, 'IDR', 'id')).toMatch(/^Rp\s?1\.500\.000$/);
		expect(formatMoney(1_500_000, 'IDR', 'en')).toMatch(/^(Rp|IDR)\s?1,500,000$/);
	});

	it('keeps the decimals a currency uses', () => {
		expect(formatMoney(12.5, 'USD', 'en')).toBe('$12.50');
	});

	it('falls back to the code for a currency it does not know', () => {
		expect(formatMoney(5, 'not a code', 'en')).toBe('5 not a code');
	});
});

describe('prices', () => {
	const plan = { price_idr: 100_000, promo_price_idr: 49_000, promo_ends_at: '2026-12-31T00:00:00Z' };
	const before = new Date('2026-11-01T00:00:00Z');
	const after = new Date('2027-01-01T00:00:00Z');

	it('uses the promo price while it runs', () => {
		expect(promoPriceLive(plan, before)).toBe(true);
		expect(currentMonthlyPrice(plan, before)).toBe(49_000);
	});

	it('goes back to the list price when the promo has ended or is no cheaper', () => {
		expect(promoPriceLive(plan, after)).toBe(false);
		expect(currentMonthlyPrice(plan, after)).toBe(100_000);
		expect(promoPriceLive({ ...plan, promo_price_idr: 100_000 }, before)).toBe(false);
		expect(promoPriceLive({ ...plan, promo_price_idr: null }, before)).toBe(false);
	});

	it('runs a promo with no end date', () => {
		expect(promoPriceLive({ ...plan, promo_ends_at: null }, after)).toBe(true);
	});

	it('divides a total over its months', () => {
		expect(perMonth(540_000, 6)).toBe(90_000);
		expect(perMonth(100, 0)).toBe(100);
	});
});

describe('plansForSale', () => {
	const plan = (id: string, over: Partial<Plan> = {}) => ({ id, is_active: true, price_idr: 50_000, ...over }) as Plan;
	it('lists priced, shown plans other than the current one', () => {
		const offer = { current_plan_id: 'a', plans: [plan('a'), plan('b'), plan('c', { price_idr: null }), plan('d', { is_active: false })] };
		expect(plansForSale(offer).map((p) => p.id)).toEqual(['b']);
	});
});

describe('buildSummaryLines', () => {
	it('shows the subtotal, the discounts that apply and the total', () => {
		expect(buildSummaryLines(quote)).toEqual([
			{ kind: 'subtotal', amount: 600_000 },
			{ kind: 'term', amount: -60_000, code: undefined },
			{ kind: 'total', amount: 540_000 },
		]);
	});

	it('adds promo, coupon and referral lines, with their codes, when they are not zero', () => {
		const lines = buildSummaryLines({ ...quote, promo_discount: 10_000, coupon_discount: 20_000, coupon_code: 'SAVE20', referral_discount: 5_000, referral_code: 'FRIEND1', amount: 505_000 });
		expect(lines.map((l) => [l.kind, l.amount, l.code])).toEqual([
			['subtotal', 600_000, undefined],
			['promo', -10_000, undefined],
			['term', -60_000, undefined],
			['coupon', -20_000, 'SAVE20'],
			['referral', -5_000, 'FRIEND1'],
			['total', 505_000, undefined],
		]);
	});

	it('shows just subtotal and total for a one-month order with nothing off', () => {
		const lines = buildSummaryLines({ ...quote, term_months: 1, base_amount: 100_000, term_discount: 0, amount: 100_000 });
		expect(lines.map((l) => l.kind)).toEqual(['subtotal', 'total']);
	});
});

describe('meterShare', () => {
	it('is the used part of the limit, capped at full', () => {
		expect(meterShare(1, 4)).toBe(0.25);
		expect(meterShare(9, 4)).toBe(1);
	});
	it('has no share when unlimited', () => {
		expect(meterShare(3, null)).toBeNull();
	});
	it('is full only when something is used against a limit of zero', () => {
		expect(meterShare(0, 0)).toBe(0);
		expect(meterShare(1, 0)).toBe(1);
	});
});
