import { describe, expect, it } from 'vitest';
import { budgetState, categoryLook, compareMonths, fillDays, fillPeriod, itemsTotal, moneyPieces, niceScale, periodLabel, shiftMonth } from './money';

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

describe('moneyPieces', () => {
	const raised = (text: string) =>
		moneyPieces(text)
			.map((p) => (p.cents ? `^${p.text}` : p.text))
			.join('');

	it('raises the two-digit cents of an amount, in either decimal style', () => {
		expect(raised('$1,234.56')).toBe('$1,234^.56');
		expect(raised('Rp 120.000,00')).toBe('Rp 120.000^,00');
		expect(raised('12,50 €')).toBe('12^,50 €');
	});

	it('raises every amount in a sentence', () => {
		expect(raised('Rp 50.000,00 over of Rp 1.000.000,00')).toBe('Rp 50.000^,00 over of Rp 1.000.000^,00');
	});

	it('leaves numbers without two-digit cents alone', () => {
		for (const text of ['Rp 1.000', '12,5 rb', '$0.0012', '18%', '3 transactions']) {
			expect(raised(text)).toBe(text);
		}
	});
});

describe('money months and receipt items', () => {
	it('fills a payday-to-payday month across the month boundary', () => {
		const days = fillPeriod('2026-10-25', '2026-11-24', [{ date: '2026-11-01', cents: 900 }]);
		expect(days).toHaveLength(31);
		expect(days[0]).toEqual({ date: '2026-10-25', day: 1, dom: 25, cents: 0 });
		expect(days.find((d) => d.date === '2026-11-01')?.cents).toBe(900);
		expect(days.at(-1)?.dom).toBe(24);
	});

	it('labels calendar months by name and other months by their dates', () => {
		expect(periodLabel('2026-10', '2026-10-01', '2026-10-31', 1)).toBe('October 2026');
		expect(periodLabel('2026-10', '2026-10-25', '2026-11-24', 25)).toBe('Oct 25 – Nov 24, 2026');
	});

	it('adds up the lines that have a name and a price', () => {
		const total = itemsTotal([
			{ name: 'Rice 5kg', quantity: '1', price: '75000' },
			{ name: 'Eggs', quantity: '2', price: '28000' },
			{ name: '', quantity: '1', price: '5' },
		]);
		expect(total.items).toHaveLength(2);
		expect(total.cents).toBe(13_100_000);
	});
});
