import { describe, expect, it } from 'vitest';
import { googleErrorFrom } from './problems';

describe('googleErrorFrom', () => {
	it('keeps the reasons it knows', () => {
		for (const reason of ['cancelled', 'unavailable', 'conflict', 'expired', 'unverified', 'invite', 'failed']) expect(googleErrorFrom(reason)).toBe(reason);
	});

	it('never shows text from the address bar: anything else is a plain failure', () => {
		expect(googleErrorFrom('<b>Call 0812 to unlock your account</b>')).toBe('failed');
		expect(googleErrorFrom(null)).toBeNull();
		expect(googleErrorFrom('')).toBeNull();
	});
});
