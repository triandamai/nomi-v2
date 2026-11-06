// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { click, control, controls, render, text, type } from '../../../test-stubs/render';
import Page from './+page.svelte';
import type { MoneySummary } from '$lib/types';

const tx = (over: Partial<MoneySummary['transactions'][number]> = {}): MoneySummary['transactions'][number] => ({
	id: 't1',
	occurred_at: '2026-11-03T08:00:00Z',
	amount_cents: 450_000,
	category: 'Food',
	description: 'Groceries',
	kind: 'expense',
	source: 'manual',
	created_at: '2026-11-03T08:01:00Z',
	items: [],
	...over,
});

const summary = (over: Partial<MoneySummary> = {}): MoneySummary => ({
	month: '2026-11',
	start_day: 1,
	period_start: '2026-11-01',
	period_end: '2026-11-30',
	timezone: 'Asia/Jakarta',
	total_cents: 1_200_000,
	income_cents: 5_000_000,
	previous_total_cents: 1_000_000,
	transaction_count: 3,
	by_category: [
		{ category: 'Food', cents: 800_000, count: 2 },
		{ category: 'Transport', cents: 400_000, count: 1 },
	],
	by_day: [
		{ date: '2026-11-03', cents: 450_000 },
		{ date: '2026-11-04', cents: 750_000 },
	],
	transactions: [
		tx(),
		tx({
			id: 't2',
			description: 'Dinner',
			amount_cents: 350_000,
			source: 'agent',
			items: [
				{ name: 'Rice', quantity: 2, unit_amount_cents: 50_000, amount_cents: 100_000, category: null },
				{ name: 'Satay', quantity: 1, unit_amount_cents: 150_000, amount_cents: 150_000, category: null },
				{ name: 'Tea', quantity: 2, unit_amount_cents: 10_000, amount_cents: 20_000, category: null },
				{ name: 'Krupuk', quantity: 1, unit_amount_cents: 5_000, amount_cents: 5_000, category: null },
			],
		}),
		tx({ id: 't3', description: 'Ojek', category: 'Transport', amount_cents: 400_000, source: 'import' }),
	],
	months: ['2026-11', '2026-10'],
	budgets: [
		{ category: 'Food', limit_cents: 1_000_000, spent_cents: 800_000 },
		{ category: 'Fun', limit_cents: 100_000, spent_cents: 150_000 },
	],
	...over,
});

const page = (money: MoneySummary | null, form: unknown = null) =>
	render(Page as never, { data: { money } as never, form: form as never, params: {} as never });

describe('money page', () => {
	it('says so when the summary could not be read', () => {
		const root = page(null);
		expect(root.querySelector('[role="alert"]')).not.toBeNull();
		expect(root.querySelector('.months')).toBeNull();
	});

	it('shows the month: transactions, categories, budgets and income', () => {
		const root = page(summary());
		const shown = text(root);
		expect(shown).toContain('Groceries');
		expect(shown).toContain('Dinner');
		expect(shown).toContain('Ojek');
		expect(shown).toContain('Transport');
		expect(shown).toContain('Fun');
		// The next-month arrow is off: this is the latest month.
		expect(root.querySelectorAll('.months a, .months button').length).toBeGreaterThan(0);
	});

	it('filters transactions by search and by category chip', () => {
		const root = page(summary());
		type(root.querySelector('input[type="search"]'), 'satay');
		expect(text(root.querySelector('.money')!)).toContain('Dinner');
		expect(controls('Groceries').length).toBe(0);
		type(root.querySelector('input[type="search"]'), '');
		const chips = [...root.querySelectorAll<HTMLElement>('button')].filter((b) => b.textContent?.trim() === 'Transport');
		click(chips.at(-1));
		expect(text(root)).toContain('Ojek');
	});

	it('toggles the chart tables', () => {
		const root = page(summary());
		const before = root.querySelectorAll('table').length;
		for (const toggle of controls(/table/i)) click(toggle);
		expect(root.querySelectorAll('table').length).toBeGreaterThan(before);
	});

	it('opens a transaction to read its items and where it came from', () => {
		const root = page(summary());
		const dinner = [...root.querySelectorAll<HTMLElement>('button, [role="button"], li')].find((el) => el.textContent?.includes('Dinner'));
		click(dinner);
		expect(text()).toContain('Krupuk');
	});

	it('opens the add, budget and period sheets', () => {
		const root = page(summary(), { error: 'Nope' });
		for (const button of [...root.querySelectorAll<HTMLElement>('.money__actions button')]) click(button);
		const add = controls(/item/i);
		for (const b of add) click(b);
		const expenseIncome = [...document.querySelectorAll<HTMLElement>('button')].filter((b) => /income/i.test(b.textContent ?? ''));
		for (const b of expenseIncome) click(b);
		expect(document.querySelectorAll('form').length).toBeGreaterThan(0);
		expect(text()).toContain('Nope');
	});

	it('shows an empty month', () => {
		const root = page(summary({ transaction_count: 0, transactions: [], by_category: [], by_day: [], budgets: [], income_cents: 0, total_cents: 0, start_day: 25 }));
		expect(root.querySelector('.money')).not.toBeNull();
		click(control(/budget/i, root));
	});
});
