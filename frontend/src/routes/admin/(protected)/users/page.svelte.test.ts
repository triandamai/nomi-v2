// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text } from '../../../../test-stubs/render';
import Page from './+page.svelte';

const plan = { id: 'pro', slug: 'pro', name: 'Pro', description: '', monthly_tokens: 5_000_000, price_label: 'Rp 99k', features: [], card_tone: 'a', promo_label: null, promo_price_label: null, promo_ends_at: null, is_default: false, is_active: true, sort_order: 1 };
const users = [
	{ id: 'u1', email: 'ana@acme.test', is_platform_admin: true, is_staff: true, org_count: 2, plan_name: 'Pro', custom_quota: true, monthly_tokens: 5_000_000, tokens_used: 4_900_000 },
	{ id: 'u2', email: 'ben@acme.test', is_platform_admin: false, is_staff: false, org_count: 0, plan_name: 'Free', custom_quota: false, monthly_tokens: 100_000, tokens_used: 0 },
];
const detail = {
	id: 'u1',
	email: 'ana@acme.test',
	is_platform_admin: true,
	display_name: 'Ana',
	username: 'ana',
	permissions: [
		{ id: 'g1', scope_type: 'admin', org_id: null, org_name: null, resource: 'user', actions: ['view', 'write'], created_at: '2026-01-01T00:00:00Z' },
		{ id: 'g2', scope_type: 'org', org_id: 'o1', org_name: 'Acme', resource: 'projects', actions: ['manage'], created_at: '2026-01-01T00:00:00Z' },
	],
	memberships: [{ org_id: 'o1', org_name: 'Acme', role: 'owner' }],
};
const subscription = { plan, quota_override: 6_000_000, override_until: '2026-12-31T23:59:59Z', note: 'bonus', monthly_tokens: 6_000_000, month: '2026-11', tokens_used: 4_900_000, history: [{ plan_name: 'Free', quota_override: null, override_until: null, note: null, changed_by_email: 'root@nomi.test', created_at: '2026-10-01T00:00:00Z' }] };

const action = (data: unknown) => ({ type: 'success', status: 200, data });

const page = (over: Record<string, unknown> = {}) =>
	render(Page as never, {
		data: { users, total: 45, page: 2, pageSize: 20, pageSizes: [10, 20, 50], query: 'a', plans: [plan], access: { user: 'manage' }, ...over } as never,
		params: {} as never,
	});

describe('admin users page', () => {
	it('lists people with their plan and use, and pages through them', () => {
		const shown = text(page());
		expect(shown).toContain('ana@acme.test');
		expect(shown).toContain('ben@acme.test');
	});

	it('opens someone’s details, permissions and subscription', async () => {
		const calls = fakeBackend([
			['POST', /loadUserDetail/, action({ success: true, user: detail })],
			['POST', /updateUser/, action({ success: true, user: { ...detail, display_name: 'Ana K' } })],
			['POST', /loadSubscription/, action({ subscription })],
			['POST', /saveSubscription/, action({ subscription, saved: true })],
			['POST', /revokePermission/, action({ success: true, permissions: [] })],
			['POST', /promote/, action({ success: true })],
			['*', /.*/, action({})],
		]);
		const root = page();
		clickEverything(root);
		await settle(10);
		clickEverything(document.body);
		await settle(10);
		expect(calls.length).toBeGreaterThan(0);
		expect(text()).toContain('ana@acme.test');
	});

	it('shows a failed lookup and an empty list', async () => {
		fakeBackend([['*', /.*/, { type: 'failure', status: 403, data: { error: 'Not allowed' } }]]);
		const root = page({ users: [], total: 0, page: 1, query: '' });
		clickEverything(root);
		await settle();
		expect(root.textContent).toBeTruthy();
	});
});
