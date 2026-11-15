import { describe, expect, it } from 'vitest';
import { busiestWeekday, mostActiveLabel } from './profileInsights';

describe('mostActiveLabel', () => {
	// 2026-11-10 is a Tuesday, 2026-11-12 a Thursday.
	const activity = [
		{ date: '2026-11-10', tokens: 900 },
		{ date: '2026-11-17', tokens: 300 },
		{ date: '2026-11-12', tokens: 1000 },
	];

	it('names the busiest weekday', () => {
		expect(busiestWeekday(activity)).toBe(2);
		expect(busiestWeekday([])).toBeNull();
		expect(busiestWeekday([{ date: '2026-11-10', tokens: 0 }])).toBeNull();
	});

	it('adds the active hours when the backend knows them', () => {
		expect(mostActiveLabel(activity, { from: '21:00', to: '23:00' }, 'en')).toBe('Tue · 21:00–23:00');
		expect(mostActiveLabel(activity, null, 'en')).toBe('Tue');
		expect(mostActiveLabel([], { from: '09:00', to: '11:00' }, 'en')).toBe('09:00–11:00');
		expect(mostActiveLabel([], null, 'en')).toBeNull();
	});
});
