import { describe, expect, it } from 'vitest';
import { formatIdr, planOffer, usdToIdr } from './planCheckout';
import type { Plan, PlansForUser } from '$lib/types';

const plan = (id: string, extra: Partial<Plan> = {}): Plan => ({
	id,
	slug: id,
	name: id,
	description: '',
	monthly_tokens: 1,
	price_label: '',
	features: [],
	card_tone: 'glow',
	promo_label: null,
	promo_price_label: null,
	promo_ends_at: null,
	is_default: false,
	is_active: true,
	sort_order: 0,
	price_idr: null,
	...extra,
});

const plans = (extra: Partial<PlansForUser> = {}): PlansForUser => ({
	plans: [],
	current_plan_id: 'free',
	monthly_tokens: 1,
	custom_quota: false,
	override_until: null,
	waitlist: [],
	checkout_available: true,
	...extra,
});

describe('what a plan offers', () => {
	it('offers nothing on the current or default plan', () => {
		expect(planOffer(plan('free'), plans())).toBe('none');
		expect(planOffer(plan('basic', { is_default: true }), plans({ current_plan_id: 'pro' }))).toBe('none');
	});

	it('sells a plan with a price while payments are set up', () => {
		expect(planOffer(plan('pro', { price_idr: 49_000 }), plans())).toBe('checkout');
		expect(planOffer(plan('pro', { price_idr: 49_000 }), plans({ checkout_available: false }))).toBe('waitlist');
	});

	it('offers the waitlist of a plan not sold yet, once', () => {
		expect(planOffer(plan('pro'), plans())).toBe('waitlist');
		expect(planOffer(plan('pro'), plans({ waitlist: ['pro'] }))).toBe('waitlisted');
	});
});

describe('rupiah', () => {
	it('writes rupiah the Indonesian way and converts dollars', () => {
		expect(formatIdr(1800)).toBe('Rp 1.800');
		expect(formatIdr(0)).toBe('Rp 0');
		expect(usdToIdr(0.11, 16_000)).toBe(1760);
	});
});
