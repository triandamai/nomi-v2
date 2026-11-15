import { afterEach, describe, expect, it } from 'vitest';
import { leaveTo, signedOutTarget } from './leaving';

afterEach(() => leaveTo(null));

describe('where a signed-out tab goes', () => {
	it('is sign in, unless the tab is closing the account on purpose', () => {
		expect(signedOutTarget()).toBe('/sign-in');
		leaveTo('/sign-in?closed=1');
		expect(signedOutTarget()).toBe('/sign-in?closed=1');
		leaveTo(null);
		expect(signedOutTarget()).toBe('/sign-in');
	});

	it('never leaves the site', () => {
		leaveTo('https://evil.test');
		expect(signedOutTarget()).toBe('/sign-in');
		leaveTo('//evil.test');
		expect(signedOutTarget()).toBe('/sign-in');
	});
});
