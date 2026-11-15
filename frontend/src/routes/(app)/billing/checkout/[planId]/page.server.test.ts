import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, textResponse } from '../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { load } = await import('./+page.server');

const plan = (id: string, over: Record<string, unknown> = {}) => ({ id, is_active: true, price_idr: 50_000, ...over });
const answer = (plans: unknown[], current = 'free') =>
	apiFetch.mockImplementation(async (_f: unknown, _c: unknown, path: string, init?: { body?: string }) => {
		if (path === '/api/plans') return json({ plans, current_plan_id: current });
		if (path === '/api/billing/terms')
			return json([
				{ id: 't6', months: 6, discount_percent: 10, label: '', is_active: true, sort_order: 2 },
				{ id: 't1', months: 1, discount_percent: 0, label: '', is_active: true, sort_order: 1 },
				{ id: 'off', months: 24, discount_percent: 30, label: '', is_active: false, sort_order: 3 },
			]);
		const { months } = JSON.parse(init?.body ?? '{}');
		return months === 6 ? textResponse('no', 400) : json({ term_months: months, amount: 50_000 });
	});

const redirected = async (planId: string) => {
	try {
		await load(event({ params: { planId } }) as never);
	} catch (thrown) {
		return thrown as { status: number; location: string };
	}
	return null;
};

beforeEach(() => apiFetch.mockReset());

describe('checkout: load', () => {
	it('loads the plan with its active terms in order, and a quote for each', async () => {
		answer([plan('pro'), plan('free', { price_idr: null })]);
		const data = (await load(event({ params: { planId: 'pro' } }) as never)) as { plan: { id: string }; terms: { months: number }[]; quotes: Record<number, unknown> };
		expect(data.plan.id).toBe('pro');
		expect(data.terms.map((t) => t.months)).toEqual([1, 6]);
		expect(data.quotes[1]).toMatchObject({ amount: 50_000 });
		expect(data.quotes[6]).toBeNull();
	});

	it('sends the person back to Billing for a plan that is unknown, free, hidden or their own', async () => {
		answer([plan('pro'), plan('free', { price_idr: null }), plan('old', { is_active: false })], 'pro');
		for (const id of ['nope', 'free', 'old', 'pro']) expect(await redirected(id)).toMatchObject({ status: 303, location: '/billing' });
	});
});
