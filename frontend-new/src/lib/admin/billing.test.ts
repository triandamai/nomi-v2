import { describe, expect, it } from 'vitest';
import { couponFromForm, money, paymentStatusLabel, termFromForm } from './billing';

const form = (fields: Record<string, string | string[]>) => {
	const data = new FormData();
	for (const [key, value] of Object.entries(fields)) for (const one of [value].flat()) data.append(key, one);
	return data;
};

describe('a billing term from its form', () => {
	it('reads six months at ten percent off, shown', () => {
		expect(termFromForm(form({ months: '6', discount_percent: '10', label: 'Hemat', is_active: 'on', sort_order: '2' }))).toEqual({
			ok: true,
			input: { months: 6, discount_percent: 10, label: 'Hemat', is_active: true, sort_order: 2 },
		});
	});

	it('refuses no months, and a discount over 100', () => {
		expect(termFromForm(form({ months: '0' })).ok).toBe(false);
		expect(termFromForm(form({ months: '3', discount_percent: '120' })).ok).toBe(false);
	});
});

describe('a coupon from its form', () => {
	it('reads the code in capitals, the limits and the days in UTC', () => {
		const read = couponFromForm(
			form({ code: 'lebaran25', discount_kind: 'percent', discount_value: '25', max_redemptions: '100', per_user_limit: '1', starts_at: '2026-03-01', ends_at: '2026-03-31', plan_ids: ['p1', 'p2'], min_months: '3', is_active: 'on' }),
		);
		expect(read).toEqual({
			ok: true,
			input: {
				code: 'LEBARAN25',
				description: null,
				discount_kind: 'percent',
				discount_value: 25,
				max_redemptions: 100,
				per_user_limit: 1,
				starts_at: '2026-03-01T00:00:00.000Z',
				ends_at: '2026-03-31T23:59:59.000Z',
				plan_ids: ['p1', 'p2'],
				min_months: 3,
				first_payment_only: false,
				is_active: true,
			},
		});
	});

	it('takes an amount in rupiah with its dots, with no cap', () => {
		const read = couponFromForm(form({ code: 'HEMAT', discount_kind: 'amount', discount_value: '50.000' }));
		expect(read.ok && read.input).toMatchObject({ discount_kind: 'amount', discount_value: 50000, max_redemptions: null, per_user_limit: 1, min_months: 1 });
	});

	it('refuses a code with a space, a percent over 100, and an end before the start', () => {
		expect(couponFromForm(form({ code: 'A B', discount_value: '5' })).ok).toBe(false);
		expect(couponFromForm(form({ code: 'AB', discount_value: '101' })).ok).toBe(false);
		expect(couponFromForm(form({ code: 'AB', discount_value: '5', starts_at: '2026-04-02', ends_at: '2026-04-01' })).ok).toBe(false);
		expect(couponFromForm(form({ code: 'AB', discount_value: '5', starts_at: '2026-02-31' })).ok).toBe(false);
	});
});

describe('money and payment states', () => {
	it('shows rupiah without decimals', () => {
		expect(money(1500000, 'IDR', 'id').replace(/\s/g, ' ')).toBe('Rp 1.500.000');
	});

	it('names each state', () => {
		expect(paymentStatusLabel('paid')).toBeTruthy();
	});
});
