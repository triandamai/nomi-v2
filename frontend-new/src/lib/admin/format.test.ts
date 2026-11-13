import { describe, expect, it } from 'vitest';
import { compact, dateTime, dayOfMonth, everyDay, full, isoFromLocal, localParts, monthLabel, niceMax, shiftMonth, slugify, usd } from './format';

describe('numbers', () => {
	it('shortens big numbers, keeping one decimal, in the person’s language', () => {
		expect(compact(1234, 'en')).toBe('1.2K');
		expect(compact(2_500_000, 'en')).toBe('2.5M');
		expect(compact(950, 'en')).toBe('950');
		expect(compact(1234, 'id')).toBe('1,2\u00a0rb');
	});

	it('writes whole numbers in full, grouped', () => {
		expect(full(1234567, 'en')).toBe('1,234,567');
		expect(full(1234567, 'id')).toBe('1.234.567');
	});

	it('shows dollars to the cent, and tiny amounts to the hundredth of a cent', () => {
		expect(usd(12.5, 'en')).toBe('$12.50');
		expect(usd(0.00042, 'en')).toBe('$0.0004');
		expect(usd(0, 'en')).toBe('$0.00');
	});
});

describe('months', () => {
	it('moves a month back and forward across years', () => {
		expect(shiftMonth('2026-01', -1)).toBe('2025-12');
		expect(shiftMonth('2026-12', 1)).toBe('2027-01');
		expect(shiftMonth('2026-11', 0)).toBe('2026-11');
	});

	it('names a month in the person’s language, in UTC', () => {
		expect(monthLabel('2026-11', 'en')).toBe('November 2026');
		expect(monthLabel('2026-11', 'id')).toBe('November 2026');
		expect(monthLabel('2026-02', 'id')).toBe('Februari 2026');
	});

	it('fills in every day of the month, zero where nothing was used', () => {
		const days = everyDay('2026-02', [
			{ date: '2026-02-03', chat_tokens: 5, embedding_tokens: 1 },
			{ date: '2026-02-28', chat_tokens: 9, embedding_tokens: 0 },
		]);
		expect(days).toHaveLength(28);
		expect(days[0]).toEqual({ date: '2026-02-01', chat_tokens: 0, embedding_tokens: 0 });
		expect(days[2]).toEqual({ date: '2026-02-03', chat_tokens: 5, embedding_tokens: 1 });
		expect(days[27].chat_tokens).toBe(9);
		expect(dayOfMonth('2026-02-03')).toBe(3);
	});

	it('rounds the chart’s top up to a round number', () => {
		expect(niceMax(0)).toBe(1);
		expect(niceMax(7)).toBe(10);
		expect(niceMax(1234)).toBe(2000);
		expect(niceMax(42_000)).toBe(50_000);
		expect(niceMax(100)).toBe(100);
	});
});

describe('dates', () => {
	it('shows a date and time in the person’s language and zone', () => {
		expect(dateTime('2026-11-10T08:15:30.123456Z', 'en', 'UTC')).toBe('10 Nov 2026, 08:15');
		expect(dateTime('2026-11-10T08:15:30Z', 'en', 'Asia/Jakarta')).toBe('10 Nov 2026, 15:15');
		expect(dateTime(null, 'en', 'UTC')).toBe('—');
	});

	it('turns a date and time typed in a zone into an instant, and back', () => {
		expect(isoFromLocal('2026-12-01', '09:30', 'Asia/Jakarta')).toBe('2026-12-01T02:30:00.000Z');
		expect(isoFromLocal('2026-12-01', '', 'UTC')).toBe('2026-12-01T00:00:00.000Z');
		expect(isoFromLocal('', '09:30', 'UTC')).toBeNull();
		expect(isoFromLocal('nope', '', 'UTC')).toBeNull();
		expect(localParts('2026-12-01T02:30:00Z', 'Asia/Jakarta')).toEqual({ date: '2026-12-01', time: '09:30' });
		expect(localParts(null, 'UTC')).toEqual({ date: '', time: '' });
	});
});

describe('slugify', () => {
	it('makes an id from a name: lower case, dashes, nothing at the ends', () => {
		expect(slugify('Pro Plus!')).toBe('pro-plus');
		expect(slugify('  Bisnis — 2026  ')).toBe('bisnis-2026');
		expect(slugify('')).toBe('');
	});
});
