import { describe, expect, it } from 'vitest';
import { budgetState, categoryLook, compareMonths, fillDays, niceScale, shiftMonth } from './money';

describe('money helpers', () => {
	it('fills every day of the month', () => {
		const days = fillDays('2026-02', [{ date: '2026-02-03', cents: 500 }]);
		expect(days).toHaveLength(28);
		expect(days[2]).toEqual({ date: '2026-02-03', day: 3, cents: 500 });
		expect(days[0].cents).toBe(0);
	});
	it('rounds the axis up to a clean maximum', () => {
		expect(niceScale(870)).toEqual({ max: 1000, ticks: [0, 250, 500, 750, 1000] });
		expect(niceScale(0)).toEqual({ max: 1, ticks: [0, 1] });
	});
	it('compares with the previous month', () => {
		expect(compareMonths(80, 100, 'September')).toEqual({ text: '20% less than September', direction: 'down' });
		expect(compareMonths(150, 100, 'September')?.direction).toBe('up');
		expect(compareMonths(10, 0, 'September')).toBeNull();
	});
	it('shifts months across years', () => {
		expect(shiftMonth('2026-01', -1)).toBe('2025-12');
		expect(shiftMonth('2026-12', 1)).toBe('2027-01');
	});
});

describe('categories and budgets', () => {
	it('gives every category a stable look', () => {
		expect(categoryLook('Food')).toEqual({ shape: 'cookie9', tone: 'citrus' });
		expect(categoryLook('pets')).toEqual(categoryLook('pets'));
	});
	it('flags budgets that are nearly or over spent', () => {
		expect(budgetState(85, 100)).toMatchObject({ nearly: true, over: false });
		expect(budgetState(120, 100)).toMatchObject({ over: true });
		expect(budgetState(10, 100)).toMatchObject({ nearly: false, over: false });
	});
});
