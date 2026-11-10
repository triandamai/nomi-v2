import { describe, expect, it } from 'vitest';
import { ago, dayPart, greeting, stamp } from './time';

const now = new Date('2026-10-21T08:14:30Z');
const before = (ms: number) => new Date(now.getTime() - ms).toISOString();
const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

describe('ago', () => {
	it('says how long ago in the sidebar’s short mono form: now, 2h, 3d', () => {
		expect(ago(before(20_000), now)).toBe('now');
		expect(ago(before(5 * MINUTE), now)).toBe('5m');
		expect(ago(before(2 * HOUR + 10 * MINUTE), now)).toBe('2h');
		expect(ago(before(2 * DAY), now)).toBe('2d');
		expect(ago(before(3 * DAY + 5 * HOUR), now)).toBe('3d');
		expect(ago(before(15 * DAY), now)).toBe('2w');
	});

	it('gives the date once it is more than a month ago', () => {
		expect(ago('2026-08-02T10:00:00Z', now)).toBe('2 Aug');
	});

	it('treats a time a little in the future (clock skew) as now', () => {
		expect(ago(new Date(now.getTime() + 30_000).toISOString(), now)).toBe('now');
	});
});

describe('dayPart', () => {
	it('reads the hour where the person is', () => {
		// 08:14 UTC is 15:14 in Jakarta and 01:14 in Los Angeles.
		expect(dayPart(now, 'UTC')).toBe('morning');
		expect(dayPart(now, 'Asia/Jakarta')).toBe('afternoon');
		expect(dayPart(now, 'America/Los_Angeles')).toBe('evening');
	});

	it('splits the day at 4, 12, 15 and 18', () => {
		const at = (hour: number) => dayPart(new Date(Date.UTC(2026, 9, 21, hour, 30)), 'UTC');
		expect([3, 4, 11, 12, 14, 15, 17, 18, 23].map(at)).toEqual(['evening', 'morning', 'morning', 'midday', 'midday', 'afternoon', 'afternoon', 'evening', 'evening']);
	});

	it('falls back to UTC for a zone it does not know', () => {
		expect(dayPart(now, 'Mars/Olympus')).toBe('morning');
	});
});

describe('greeting', () => {
	it('greets the person by their first name, as the pen does', () => {
		expect(greeting('Mario', 'morning')).toBe('Morning, Mario.');
		expect(greeting('Mario', 'midday')).toBe('Afternoon, Mario.');
		expect(greeting('Mario', 'evening')).toBe('Evening, Mario.');
	});
});

describe('stamp', () => {
	it('is the top bar’s date and time where the person is', () => {
		expect(stamp(now, 'en', 'UTC')).toBe('Wed, 21 Oct 2026 · 08:14');
		expect(stamp(now, 'en', 'Asia/Jakarta')).toBe('Wed, 21 Oct 2026 · 15:14');
	});

	it('names the day and month in Indonesian', () => {
		expect(stamp(now, 'id', 'Asia/Jakarta')).toBe('Rab, 21 Okt 2026 · 15:14');
	});
});
