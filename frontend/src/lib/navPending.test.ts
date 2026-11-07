import { describe, expect, it } from 'vitest';
import { opensPage } from './navPending';

describe('opensPage', () => {
	const to = new URL('http://localhost/register?redirect_to=%2Fx');

	it('matches the link whose page is loading, by path and search', () => {
		expect(opensPage('/register?redirect_to=%2Fx', to)).toBe(true);
		expect(opensPage('/register?redirect_to=%2Fx#top', to)).toBe(true);
		expect(opensPage('/register', to)).toBe(false);
		expect(opensPage('/login', to)).toBe(false);
	});

	it('is false with nothing loading or for another site', () => {
		expect(opensPage('/register', null)).toBe(false);
		expect(opensPage('https://example.com/register?redirect_to=%2Fx', to)).toBe(false);
	});
});
