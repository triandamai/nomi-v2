import { describe, expect, it } from 'vitest';
import { inviteCodeFrom, withRedirect } from './redirects';

describe('inviteCodeFrom', () => {
	it('finds the invite a sign-up or sign-in is on its way to', () => {
		expect(inviteCodeFrom('/invite/Ab12Cd34')).toBe('Ab12Cd34');
		expect(inviteCodeFrom('/invite/Ab12Cd34?x=1')).toBe('Ab12Cd34');
	});

	it('finds none anywhere else', () => {
		for (const target of [null, undefined, '', '/', '/invite/', '/projects/1', '//invite/x', 'https://evil.test/invite/x', '/invite/a/b']) expect(inviteCodeFrom(target)).toBeNull();
	});
});

describe('withRedirect', () => {
	it('carries where the person was going to the next page', () => {
		expect(withRedirect('/register', '/invite/abc')).toBe('/register?redirect_to=%2Finvite%2Fabc');
		expect(withRedirect('/sign-in', '/chat/1', { email: 'ana@example.com' })).toBe('/sign-in?redirect_to=%2Fchat%2F1&email=ana%40example.com');
	});

	it('leaves it out when there is nowhere special to go', () => {
		expect(withRedirect('/sign-in', null)).toBe('/sign-in');
		expect(withRedirect('/sign-in', '/')).toBe('/sign-in');
		expect(withRedirect('/sign-in', 'https://evil.test')).toBe('/sign-in');
		expect(withRedirect('/register', undefined, { email: '' })).toBe('/register');
	});
});
