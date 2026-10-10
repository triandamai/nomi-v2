import { describe, expect, it } from 'vitest';
import {
	addMonths,
	clampDate,
	dateFieldOrder,
	daysInMonth,
	formatTime,
	from12Hour,
	isoFrom,
	monthGrid,
	orderRange,
	parseISODate,
	parseTime,
	to12Hour,
	toISODate,
	uses12Hour,
} from './dates';

describe('dates', () => {
	it('reads and writes plain days', () => {
		expect(toISODate(new Date(2026, 0, 5))).toBe('2026-01-05');
		expect(parseISODate('2026-02-29')).toBeNull();
		expect(parseISODate('2028-02-29')?.getDate()).toBe(29);
		expect(parseISODate('nope')).toBeNull();
		expect(isoFrom(2026, 1, 31)).toBe('2026-02-28');
	});

	it('walks months across years', () => {
		expect(addMonths(2026, 11, 1)).toEqual({ year: 2027, month: 0 });
		expect(addMonths(2026, 0, -1)).toEqual({ year: 2025, month: 11 });
		expect(daysInMonth(2026, 1)).toBe(28);
	});

	it('lays a month out in six weeks from the right weekday', () => {
		// October 2026 starts on a Thursday.
		const sunday = monthGrid(2026, 9, 0);
		expect(sunday).toHaveLength(42);
		expect(sunday[0]).toEqual({ iso: '2026-09-27', day: 27, inMonth: false });
		expect(sunday[4]).toEqual({ iso: '2026-10-01', day: 1, inMonth: true });
		expect(monthGrid(2026, 9, 1)[3].iso).toBe('2026-10-01');
	});

	it('keeps picks in bounds and ranges in order', () => {
		expect(clampDate('2026-10-12', undefined, '2026-10-10')).toBe('2026-10-10');
		expect(clampDate('2026-01-01', '2026-02-01')).toBe('2026-02-01');
		expect(orderRange('2026-10-12', '2026-10-01')).toEqual(['2026-10-01', '2026-10-12']);
	});

	it('handles both kinds of clock', () => {
		expect(parseTime('09:05')).toEqual({ hour: 9, minute: 5 });
		expect(parseTime('24:00')).toBeNull();
		expect(formatTime(7, 3)).toBe('07:03');
		expect(to12Hour(0)).toEqual({ hour: 12, pm: false });
		expect(to12Hour(13)).toEqual({ hour: 1, pm: true });
		expect(from12Hour(12, false)).toBe(0);
		expect(from12Hour(12, true)).toBe(12);
		expect(uses12Hour('en-US')).toBe(true);
		expect(uses12Hour('id')).toBe(false);
	});

	it('knows which way round dates are written', () => {
		expect(dateFieldOrder('en-US')).toEqual(['month', 'day', 'year']);
		expect(dateFieldOrder('id')).toEqual(['day', 'month', 'year']);
	});
});
