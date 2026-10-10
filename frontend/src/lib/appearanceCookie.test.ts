import { describe, expect, it } from 'vitest';
import { appearanceAttributes, formatAppearance, parseAppearance } from './appearanceCookie';

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
		expect(appearanceAttributes(null)).toBe('');
	});
});
