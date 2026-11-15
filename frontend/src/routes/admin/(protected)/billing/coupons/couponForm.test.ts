import { describe, expect, it } from 'vitest';
import { readCoupon } from './couponForm';

function form(entries: [string, string][]) {
	const data = new FormData();
	for (const [key, value] of entries) data.append(key, value);
	return data;
}

const base: [string, string][] = [
	['code', 'save20'],
	['discount_kind', 'percent'],
	['discount_value', '20'],
	['per_user_limit', '1'],
	['min_months', '1'],
	['is_active', 'true'],
];

describe('readCoupon', () => {
	it('reads a percent coupon with no cap, dates or plans', () => {
		expect(readCoupon(form(base))).toEqual({
			ok: true,
			payload: {
				code: 'SAVE20',
				description: null,
				discount_kind: 'percent',
				discount_value: 20,
				max_redemptions: null,
				per_user_limit: 1,
				starts_at: null,
				ends_at: null,
				plan_ids: [],
				min_months: 1,
				first_payment_only: false,
				is_active: true,
			},
		});
	});

	it('reads a capped amount coupon limited to plans and months', () => {
		const result = readCoupon(
			form([
				...base.filter(([k]) => k !== 'discount_kind' && k !== 'discount_value' && k !== 'min_months' && k !== 'per_user_limit'),
				['discount_kind', 'amount'],
				['discount_value', '50000'],
				['description', ' Launch '],
				['max_redemptions', '100'],
				['per_user_limit', '2'],
				['min_months', '6'],
				['plan_ids', 'p1'],
				['plan_ids', 'p2'],
				['first_payment_only', 'true'],
			]),
		);
		expect(result).toMatchObject({
			ok: true,
			payload: { discount_kind: 'amount', discount_value: 50_000, description: 'Launch', max_redemptions: 100, per_user_limit: 2, min_months: 6, plan_ids: ['p1', 'p2'], first_payment_only: true },
		});
	});

	it('turns a chosen day into the start and the end of that day', () => {
		const result = readCoupon(form([...base, ['starts_at', '2026-12-01'], ['ends_at', '2026-12-31']]));
		expect(result.ok && result.payload.starts_at).toBe(new Date('2026-12-01T00:00:00').toISOString());
		expect(result.ok && result.payload.ends_at).toBe(new Date('2026-12-31T23:59:59').toISOString());
	});

	it('names what is wrong', () => {
		const without = (name: string, ...more: [string, string][]) => form([...base.filter(([k]) => k !== name), ...more]);
		expect(readCoupon(without('code'))).toEqual({ ok: false, problem: 'code' });
		expect(readCoupon(without('code', ['code', 'two words']))).toEqual({ ok: false, problem: 'code' });
		expect(readCoupon(without('discount_value', ['discount_value', '101']))).toEqual({ ok: false, problem: 'value' });
		expect(readCoupon(without('discount_value', ['discount_value', '0']))).toEqual({ ok: false, problem: 'value' });
		expect(readCoupon(form([...base, ['max_redemptions', '0']]))).toEqual({ ok: false, problem: 'limits' });
		expect(readCoupon(without('per_user_limit', ['per_user_limit', '0']))).toEqual({ ok: false, problem: 'limits' });
		expect(readCoupon(form([...base, ['starts_at', '2026-12-31'], ['ends_at', '2026-12-01']]))).toEqual({ ok: false, problem: 'dates' });
	});
});
