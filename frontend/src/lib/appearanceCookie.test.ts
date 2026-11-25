import { describe, expect, it } from 'vitest';
import { appearanceAttributes, formatAppearance, parseAppearance, themeColorMeta } from './appearanceCookie';

describe('appearance cookie', () => {
	it('round-trips a mode and theme', () => {
		expect(parseAppearance(formatAppearance('dark', 'coral-reef'))).toEqual({ theme: 'dark', color: 'coral-reef' });
		expect(appearanceAttributes({ theme: 'dark', color: 'coral-reef' })).toBe('data-theme="dark" data-color="coral-reef"');
	});

	it('ignores anything that could break out of the attribute', () => {
		expect(parseAppearance('dark.x" onload="alert(1)')).toBeNull();
		expect(parseAppearance('dark')).toBeNull();
		expect(parseAppearance(undefined)).toBeNull();
		expect(formatAppearance('dark', '<script>')).toBeNull();
	});

	it('starts people with no saved appearance on Lagoon', () => {
		expect(appearanceAttributes(null)).toBe('data-color="lagoon"');
	});

	it('emits theme-color meta tags matching the chosen mode and colour palette', () => {
		expect(themeColorMeta({ theme: 'dark', color: 'coral-reef' })).toBe('<meta name="theme-color" content="#191211" />');
		expect(themeColorMeta({ theme: 'light', color: 'coral-reef' })).toBe('<meta name="theme-color" content="#fff8f7" />');
		const system = themeColorMeta({ theme: 'system', color: 'canopy' });
		expect(system).toContain('content="#f4f7f2" media="(prefers-color-scheme: light)"');
		expect(system).toContain('content="#0d1511" media="(prefers-color-scheme: dark)"');
	});
});
