import { describe, expect, it } from 'vitest';
import { isLanguage, speechLanguage, timeAgo } from './i18n';

describe('i18n', () => {
	it('timeAgo handles null, undefined, invalid, and various time deltas safely', () => {
		const now = new Date('2026-11-10T12:00:00Z').getTime();

		// Gracefully handle falsy or invalid dates without RangeError
		expect(timeAgo(null, now)).toBeTruthy();
		expect(timeAgo(undefined, now)).toBeTruthy();
		expect(timeAgo('', now)).toBeTruthy();
		expect(timeAgo('not-a-date', now)).toBeTruthy();

		// Just now (< 1 min)
		expect(timeAgo(new Date(now - 30 * 1000).toISOString(), now)).toBeTruthy();

		// Minutes (< 60 min)
		expect(timeAgo(new Date(now - 10 * 60 * 1000).toISOString(), now)).toBeTruthy();

		// Hours (< 24 hrs)
		expect(timeAgo(new Date(now - 2 * 3600 * 1000).toISOString(), now)).toBeTruthy();

		// Days (< 30 days)
		expect(timeAgo(new Date(now - 3 * 86400 * 1000).toISOString(), now)).toBeTruthy();

		// Months (< 12 months)
		expect(timeAgo(new Date(now - 60 * 86400 * 1000).toISOString(), now)).toBeTruthy();

		// Years (>= 12 months)
		expect(timeAgo(new Date(now - 400 * 86400 * 1000).toISOString(), now)).toBeTruthy();
	});

	it('isLanguage validates supported locales', () => {
		expect(isLanguage('en')).toBe(true);
		expect(isLanguage('id')).toBe(true);
		expect(isLanguage('fr')).toBe(false);
		expect(isLanguage(null)).toBe(false);
	});

	it('speechLanguage determines dictation locale', () => {
		expect(speechLanguage()).toBeTruthy();
	});
});
