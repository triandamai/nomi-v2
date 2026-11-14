// @vitest-environment jsdom
// Admin → Billing: the index and the terms, coupons and payments pages, with what their loads return.
import { describe, expect, it, vi } from 'vitest';
import { clickEverything, click, control, render, settle, text, type } from '../../../../test-stubs/render';
import Index from './+page.svelte';
import Terms from './terms/+page.svelte';
import Coupons from './coupons/+page.svelte';
import Payments from './payments/+page.svelte';

const goto = vi.fn();
vi.mock('$app/navigation', () => ({ goto: (...args: unknown[]) => goto(...args), invalidateAll: async () => {} }));

const manage = { plans: 'manage', configuration: 'manage' };
const page = (component: unknown, data: Record<string, unknown>, form: unknown = null) =>
	render(component as never, { data: { access: manage, ...data } as never, form: form as never, params: {} as never });

async function tryAll(root: HTMLElement) {
	clickEverything(root);
	await settle(4);
	clickEverything(document.body);
	await settle(3);
}

const payment = { id: 'p1', plan_id: 'pro', plan_name: 'Pro', term_months: 6, kind: 'new', base_amount: 600_000, term_discount: 60_000, promo_discount: 0, coupon_discount: 0, referral_discount: 0, amount: 540_000, currency: 'IDR', status: 'paid', invoice_url: null, expires_at: null, paid_at: '2026-11-10T00:00:00Z', period_start: null, period_end: null, payment_method: 'QRIS', created_at: '2026-11-09T00:00:00Z', user_email: 'ana@acme.test' };

describe('admin billing', () => {
	it('the index links to each page, and to the settings only for someone who can see Configuration', () => {
		const hrefs = (root: HTMLElement) => [...root.querySelectorAll('a')].map((a) => a.getAttribute('href'));
		expect(hrefs(page(Index, {}))).toEqual(['/admin/billing/terms', '/admin/billing/coupons', '/admin/billing/payments', '/admin/configuration/billing']);
		document.body.innerHTML = '';
		expect(hrefs(page(Index, { access: { plans: 'view' } }))).toEqual(['/admin/billing/terms', '/admin/billing/coupons', '/admin/billing/payments']);
	});

	it('terms: lists them, gives the example, and opens the editor', async () => {
		const terms = [
			{ id: 't1', months: 1, discount_percent: 0, label: '', is_active: true, sort_order: 1 },
			{ id: 't6', months: 6, discount_percent: 10, label: 'Half year', is_active: true, sort_order: 2 },
			{ id: 't24', months: 24, discount_percent: 30, label: '', is_active: false, sort_order: 3 },
		];
		const root = page(Terms, { terms }, { error: 'there’s already a term for that many months' });
		const shown = text(root);
		expect(shown).toContain('6 months, 10% off');
		expect(shown).toContain('Half year');
		expect(shown).toContain('10% off');
		expect(shown).toContain('already a term');
		await tryAll(root);
		expect(page(Terms, { terms: [] }).textContent).toContain('No terms');
	});

	it('coupons: lists them, searches by code and shows the backend’s refusal', async () => {
		const coupon = { id: 'c1', code: 'SAVE20', description: 'Launch', discount_kind: 'percent', discount_value: 20, max_redemptions: 100, per_user_limit: 1, starts_at: '2026-11-01T00:00:00Z', ends_at: '2026-12-31T00:00:00Z', plan_ids: ['pro'], min_months: 6, first_payment_only: true, is_active: true, redemptions: 7 };
		const coupons = [coupon, { ...coupon, id: 'c2', code: 'FLAT50', discount_kind: 'amount', discount_value: 50_000, max_redemptions: null, plan_ids: [], starts_at: null, ends_at: null, is_active: false }];
		const root = page(Coupons, { coupons, plans: [{ id: 'pro', name: 'Pro' }] }, { error: 'this coupon has been used; deactivate it instead' });
		const shown = text(root);
		expect(shown).toContain('Used 7 of 100');
		expect(shown).toContain('Only Pro');
		expect(shown).toContain('this coupon has been used');
		type(root.querySelector('input[type="search"]'), 'flat');
		expect(text(root)).not.toContain('SAVE20');
		expect(text(root)).toContain('FLAT50');
		type(root.querySelector('input[type="search"]'), '');
		await tryAll(root);
		expect(page(Coupons, { coupons: [], plans: [] }).textContent).toContain('No coupons');
	});

	it('payments: lists them with their status, and filters and pages through the URL', async () => {
		const root = page(Payments, { items: [payment, { ...payment, id: 'p2', status: 'failed', user_email: null, paid_at: null }], total: 45, page: 2, page_size: 20, status: '' });
		const shown = text(root);
		expect(shown).toContain('ana@acme.test');
		expect(shown).toContain('QRIS');
		expect(shown).toContain('Failed');
		expect(shown).toContain('45 payments');
		click(control('Pending', root));
		expect(goto).toHaveBeenCalledWith('?status=pending');
		expect(root.querySelector('a[href="?page=3"]')).not.toBeNull();
		expect(page(Payments, { items: [], total: 0, page: 1, page_size: 20, status: 'paid' }).textContent).toContain('No payments match');
	});
});
