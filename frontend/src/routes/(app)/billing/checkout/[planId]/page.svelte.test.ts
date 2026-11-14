// @vitest-environment jsdom
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { click, control, fakeBackend, json, render, settle, text, type } from '../../../../../test-stubs/render';
import Checkout from './+page.svelte';

const goto = vi.fn();
vi.mock('$app/navigation', () => ({ goto: (...args: unknown[]) => goto(...args), invalidateAll: async () => {} }));

const plan = { id: 'pro-id', slug: 'pro', name: 'Pro', description: 'More of everything', monthly_tokens: 5_000_000, price_label: '', features: ['Projects', 'Voice'], card_tone: 'violet', promo_label: null, promo_price_label: null, promo_ends_at: null, price_amount: 100_000, currency: 'IDR', promo_price_amount: null, is_default: false, is_active: true, sort_order: 2, max_projects: 10, max_orgs: null, max_org_members: 5, max_custom_crew: 1, max_storage_mb: 2048, custom_domain: true, sandbox_size: null, all_models: true, model_ids: [] };
const quote = (months: number, discount: number, over: Record<string, unknown> = {}) => {
	const base = 100_000 * months;
	return { plan_id: 'pro-id', term_months: months, currency: 'IDR', monthly_price: 100_000, base_amount: base, promo_discount: 0, term_discount: discount, coupon_discount: 0, referral_discount: 0, amount: base - discount, coupon_code: null, referral_code: null, ...over };
};
const terms = [
	{ id: 't1', months: 1, discount_percent: 0, label: '', is_active: true, sort_order: 1 },
	{ id: 't6', months: 6, discount_percent: 10, label: '', is_active: true, sort_order: 2 },
];
const data = { plan, terms, quotes: { 1: quote(1, 0), 6: quote(6, 60_000) } };
const mount = () => render(Checkout as never, { data: data as never, params: { planId: 'pro-id' } as never });

beforeEach(() => goto.mockReset());

describe('checkout', () => {
	it('shows the plan, each term with its price, and the order summary', () => {
		const shown = text(mount());
		expect(shown).toContain('Pro');
		expect(shown).toContain('Projects');
		expect(shown).toContain('10 projects');
		expect(shown).toContain('6 months');
		expect(shown).toContain('save 10%');
		expect(shown).toMatch(/Rp\s?100\.000|Rp\s?100,000/);
		expect(shown).toContain('Renews every month');
	});

	it('recomputes the summary for another term from its quote', async () => {
		const root = mount();
		click(root.querySelector('input[value="6"]'));
		await settle();
		expect(text(root)).toContain('Term discount');
		expect(text(root)).toContain('Renews every 6 months');
	});

	it('applies a code, shows its chip and takes it off again', async () => {
		const calls = fakeBackend([['POST', '/billing/quote', (body: unknown) => json(quote(1, 0, { coupon_discount: 20_000, amount: 80_000, coupon_code: (body as { code: string }).code }))]]);
		const root = mount();
		type(root.querySelector('input[type="text"]'), 'save20');
		click(control('Apply', root));
		await settle();
		expect(calls[0].body).toEqual({ plan_id: 'pro-id', months: 1, code: 'save20' });
		expect(text(root)).toContain('save20 applied');
		expect(text(root)).toContain('Coupon');
		click(control('Remove save20', root));
		await settle();
		expect(text(root)).not.toContain('applied');
	});

	it('shows the backend’s message when a code is refused', async () => {
		fakeBackend([['POST', '/billing/quote', new Response('that code has ended', { status: 400 })]]);
		const root = mount();
		type(root.querySelector('input[type="text"]'), 'old');
		click(control('Apply', root));
		await settle();
		expect(root.querySelector('[role="alert"]')?.textContent).toContain('that code has ended');
	});

	it('goes to Billing when the order is already paid, and says why when payment is unavailable', async () => {
		fakeBackend([['POST', '/billing/pay', json({ payment_id: 'pay-1', status: 'paid', invoice_url: null })]]);
		const root = mount();
		click(control('Pay', root));
		await settle();
		expect(goto).toHaveBeenCalledWith('/billing?payment=pay-1');

		fakeBackend([['POST', '/billing/pay', new Response('payments are not set up', { status: 503 })]]);
		click(control('Pay', root));
		await settle();
		expect(root.querySelector('[role="alert"]')?.textContent).toContain('payments are not set up');
	});
});
