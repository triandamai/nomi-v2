import { describe, expect, it } from 'vitest';
import { clock, dayKey, dayLabel, elapsedLabel } from './dates';

const now = new Date('2026-10-21T08:14:00Z');

describe('dayLabel', () => {
	it('says today and yesterday in words, where the person is', () => {
		expect(dayLabel('2026-10-21T01:00:00Z', 'en', now, 'Asia/Jakarta')).toBe('Today');
		expect(dayLabel('2026-10-20T10:00:00Z', 'en', now, 'Asia/Jakarta')).toBe('Yesterday');
		expect(dayLabel('2026-10-20T10:00:00Z', 'id', now, 'Asia/Jakarta')).toBe('Kemarin');
	});

	it('names an older day by weekday and date, with the year once it is another', () => {
		expect(dayLabel('2026-10-17T10:00:00Z', 'en', now, 'UTC')).toBe('Sat 17 Oct');
		expect(dayLabel('2025-12-30T10:00:00Z', 'en', now, 'UTC')).toBe('Tue 30 Dec 2025');
	});

	it('counts days in the zone given, not the server’s', () => {
		// 23:30 UTC on the 20th is already the 21st in Jakarta.
		expect(dayLabel('2026-10-20T23:30:00Z', 'en', now, 'Asia/Jakarta')).toBe('Today');
		expect(dayLabel('2026-10-20T23:30:00Z', 'en', now, 'UTC')).toBe('Yesterday');
	});
});

describe('dayKey', () => {
	it('is the calendar day in the zone', () => {
		expect(dayKey('2026-10-20T23:30:00Z', 'Asia/Jakarta')).toBe('2026-10-21');
		expect(dayKey('2026-10-20T23:30:00Z', 'UTC')).toBe('2026-10-20');
		expect(dayKey('2026-10-20T23:30:00Z', 'Not/AZone')).toBe('2026-10-20');
	});
});

describe('clock', () => {
	it('is the 24-hour time in the zone', () => {
		expect(clock('2026-10-21T07:02:00Z', 'Asia/Jakarta')).toBe('14:02');
		expect(clock('2026-10-21T07:02:00Z', undefined)).toBe('07:02');
	});
});

describe('elapsedLabel', () => {
	it('is minutes and seconds, as the live rail counts', () => {
		expect(elapsedLabel(0)).toBe('00:00');
		expect(elapsedLabel(4_200)).toBe('00:04');
		expect(elapsedLabel(65_000)).toBe('01:05');
	});
});
