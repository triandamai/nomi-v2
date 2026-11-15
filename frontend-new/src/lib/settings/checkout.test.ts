import { describe, expect, it } from 'vitest';
import { paymentOutcome, planChoice, roomOf } from './checkout';

const plan = (over: Partial<Parameters<typeof planChoice>[0]> = {}) => ({ id: 'pro', is_default: false, monthly_tokens: 10_000_000, price_idr: 99_000, ...over });
const ctx = (over: Partial<Parameters<typeof planChoice>[1]> = {}) => ({ currentId: 'free', currentTokens: 1_000_000, waitlist: [], checkoutAvailable: true, ...over });

describe('what a plan offers the person', () => {
	it('is theirs, to buy above theirs, or closed while payments aren’t set up', () => {
		expect(planChoice(plan({ id: 'free' }), ctx())).toBe('theirs');
		expect(planChoice(plan(), ctx())).toBe('buy');
		expect(planChoice(plan(), ctx({ checkoutAvailable: false }))).toBe('closed');
	});

	it('offers the waitlist for a plan without a price, and says when they’re on it', () => {
		expect(planChoice(plan({ price_idr: null }), ctx())).toBe('waitlist');
		expect(planChoice(plan({ price_idr: null }), ctx({ waitlist: ['pro'] }))).toBe('waiting');
	});

	it('offers nothing for a smaller plan, or the free one', () => {
		expect(planChoice(plan({ monthly_tokens: 500_000 }), ctx())).toBe('none');
		expect(planChoice(plan({ is_default: true, price_idr: null, id: 'basic' }), ctx())).toBe('none');
	});
});

describe('how much more room a plan without a price gives', () => {
	it('is its allowance against theirs, as the pen’s “10×”, or nothing when it isn’t more', () => {
		expect(roomOf(10_000_000, 1_000_000)).toBe('10×');
		expect(roomOf(2_500_000, 1_000_000)).toBe('2.5×');
		expect(roomOf(1_000_000, 1_000_000)).toBeNull();
		expect(roomOf(10_000_000, null)).toBeNull();
		expect(roomOf(10_000_000, 0)).toBeNull();
	});
});

describe('what a payment came to', () => {
	it('says each outcome, and only a pending one can be opened again', () => {
		expect(paymentOutcome({ status: 'paid', invoice_url: 'https://pay.example/1' })).toEqual({ kind: 'success', open: false });
		expect(paymentOutcome({ status: 'pending', invoice_url: 'https://pay.example/1' })).toEqual({ kind: 'info', open: true });
		expect(paymentOutcome({ status: 'pending', invoice_url: null })).toEqual({ kind: 'info', open: false });
		expect(paymentOutcome({ status: 'expired', invoice_url: 'https://pay.example/1' })).toEqual({ kind: 'warning', open: false });
		expect(paymentOutcome({ status: 'failed', invoice_url: null })).toEqual({ kind: 'error', open: false });
		expect(paymentOutcome({ status: 'cancelled', invoice_url: null })).toEqual({ kind: 'warning', open: false });
	});

	it('never offers to open a page that isn’t https', () => {
		expect(paymentOutcome({ status: 'pending', invoice_url: 'javascript:alert(1)' }).open).toBe(false);
	});
});
