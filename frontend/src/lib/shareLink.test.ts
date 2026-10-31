import { describe, expect, it } from 'vitest';
import { expiryLabel, nextExpiry, plainText, shareUrl } from './shareLink';

describe('share links', () => {
	it('cycles the expiry from a week to a month, a day and never', () => {
		expect([nextExpiry(7), nextExpiry(30), nextExpiry(1), nextExpiry(null)]).toEqual([30, 1, null, 7]);
	});

	it('says when the link expires', () => {
		expect([expiryLabel(7), expiryLabel(1), expiryLabel(null)]).toEqual(['in 7 days', 'in a day', 'never']);
	});

	it('links to the read-only page', () => {
		expect(shareUrl('https://nomi.app', 'abc')).toBe('https://nomi.app/shared/abc');
	});

	it('shows a reply as plain text in the preview', () => {
		expect(plainText('You spent **Rp •••** on `food`, see [the docs](https://x.y).\n\n- one\n# Two')).toBe('You spent Rp ••• on food, see the docs. one Two');
	});
});
