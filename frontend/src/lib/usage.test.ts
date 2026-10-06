import { describe, expect, it } from 'vitest';
import { formatShare, formatTokens, formatUsd, usageShare } from './usage';

describe('usage formatting', () => {
	it('shortens token counts and dollar amounts', () => {
		expect(formatTokens(1_250_000)).toBe('1.3M');
		expect(formatTokens(950)).toBe('950');
		expect(formatUsd(5.1)).toBe('$5.10');
		expect(formatUsd(0.0081)).toBe('$0.0081');
		expect(formatUsd(0)).toBe('$0.00');
	});

	it('reads the share of the allowance, never hiding a little use as 0%', () => {
		expect(usageShare(250_000, 1_000_000)).toBe(0.25);
		expect(usageShare(5, 0)).toBe(0);
		expect(formatShare(0.25)).toBe('25%');
		expect(formatShare(0.001)).toBe('1%');
		expect(formatShare(0)).toBe('0%');
	});
});
