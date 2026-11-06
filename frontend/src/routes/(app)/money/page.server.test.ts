import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, textResponse } from '../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => ({ path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(init.body) : undefined }));

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async () => json({ ok: true }));
});

describe('money: load', () => {
	it('asks for the chosen month only when it looks like one', async () => {
		apiFetch.mockImplementation(async () => json({ month: '2026-10' }));
		expect(await load(event({ url: 'http://localhost/money?month=2026-10' }))).toEqual({ money: { month: '2026-10' } });
		await load(event({ url: 'http://localhost/money?month=../../etc' }));
		expect(calls().map((c) => c.path)).toEqual(['/api/money?month=2026-10', '/api/money']);
	});

	it('gives the page nothing to show when the API fails', async () => {
		apiFetch.mockResolvedValue(json({}, 500));
		expect(await load(event())).toEqual({ money: null });
	});
});

describe('money: addTransaction', () => {
	const add = (form: Record<string, string>) => actions.addTransaction(event({ form }));

	it('sends an expense with an amount', async () => {
		expect(await add({ kind: 'expense', amount: '45000', category: ' Food ', description: ' Lunch ', occurred_at: '2026-11-03' })).toEqual({ saved: 'expense' });
		expect(calls()[0]).toEqual({
			path: '/api/money/transactions',
			method: 'POST',
			body: { kind: 'expense', amount: 45000, category: 'Food', description: 'Lunch', occurred_at: '2026-11-03' },
		});
	});

	it('lets items set the total and ignores a typed amount', async () => {
		const items = [{ name: 'Rice', quantity: 2, unit_amount: 5000 }];
		expect(await add({ kind: 'income', amount: '1', items_json: JSON.stringify(items), category: 'Gift', description: 'Rice' })).toEqual({ saved: 'income' });
		expect(calls()[0].body).toEqual({ kind: 'income', items, category: 'Gift', description: 'Rice' });
	});

	it('treats broken item JSON as no items, then refuses a missing amount', async () => {
		const result = (await add({ items_json: '{nope', category: 'Food', description: 'x' })) as { status: number };
		expect(result.status).toBe(400);
		expect(apiFetch).not.toHaveBeenCalled();
	});

	it('refuses a transaction without a description or category', async () => {
		expect(((await add({ amount: '10', category: '', description: 'x' })) as { status: number }).status).toBe(400);
		expect(((await add({ amount: '10', category: 'x', description: '' })) as { status: number }).status).toBe(400);
	});

	it("passes the API's short error through, and a fallback for a long one", async () => {
		apiFetch.mockResolvedValueOnce(textResponse('category too long', 422));
		expect(await add({ amount: '10', category: 'x', description: 'y' })).toMatchObject({ status: 422, data: { error: 'category too long' } });
		apiFetch.mockResolvedValueOnce(textResponse('x'.repeat(500), 500));
		const failed = (await add({ amount: '10', category: 'x', description: 'y' })) as { data: { error: string } };
		expect(failed.data.error).not.toContain('xxxx');
		apiFetch.mockResolvedValueOnce(textResponse('', 500));
		expect(((await add({ amount: '10', category: 'x', description: 'y' })) as { data: { error: string } }).data.error).toBeTruthy();
	});
});

describe('money: period and budgets', () => {
	it('accepts a start day from 1 to 28 only', async () => {
		for (const bad of ['0', '29', '1.5', 'x']) {
			expect(((await actions.setPeriod(event({ form: { start_day: bad } }))) as { status: number }).status).toBe(400);
		}
		expect(await actions.setPeriod(event({ form: { start_day: '25' } }))).toEqual({ saved: 'period' });
		expect(calls()).toEqual([{ path: '/api/money/period', method: 'PUT', body: { start_day: 25 } }]);
		apiFetch.mockResolvedValueOnce(textResponse('nope', 500));
		expect(await actions.setPeriod(event({ form: { start_day: '2' } }))).toMatchObject({ status: 500 });
	});

	it('sets a budget with a category and a positive limit', async () => {
		expect(((await actions.setBudget(event({ form: { category: ' ', monthly_limit: '10' } }))) as { status: number }).status).toBe(400);
		expect(((await actions.setBudget(event({ form: { category: 'Food', monthly_limit: '0' } }))) as { status: number }).status).toBe(400);
		expect(await actions.setBudget(event({ form: { category: 'Food', monthly_limit: '100000' } }))).toEqual({ saved: 'budget' });
		expect(calls()[0]).toEqual({ path: '/api/money/budgets', method: 'PUT', body: { category: 'Food', monthly_limit: 100000 } });
		apiFetch.mockResolvedValueOnce(textResponse('down', 503));
		expect(await actions.setBudget(event({ form: { category: 'Food', monthly_limit: '1' } }))).toMatchObject({ status: 503 });
	});

	it('removes a budget, treating an already-gone one as removed', async () => {
		expect(await actions.deleteBudget(event({ form: { category: 'Food & Drink' } }))).toEqual({ saved: 'budget-removed' });
		expect(calls()[0]).toMatchObject({ path: '/api/money/budgets/Food%20%26%20Drink', method: 'DELETE' });
		apiFetch.mockResolvedValueOnce(textResponse('gone', 404));
		expect(await actions.deleteBudget(event({ form: { category: 'x' } }))).toEqual({ saved: 'budget-removed' });
		apiFetch.mockResolvedValueOnce(textResponse('boom', 500));
		expect(await actions.deleteBudget(event({ form: { category: 'x' } }))).toMatchObject({ status: 500 });
	});
});
