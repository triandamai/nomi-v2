import { describe, expect, it } from 'vitest';
import { THEME_COOKIE, parseTheme, themeAttribute, withTheme } from './theme';

describe('theme', () => {
	it('has its own cookie, separate from the old app’s', () => {
		expect(THEME_COOKIE).toBe('nm_theme');
	});

	it('reads light, dark or system from the cookie, defaulting to system', () => {
		expect(parseTheme('light')).toBe('light');
		expect(parseTheme('dark')).toBe('dark');
		expect(parseTheme('system')).toBe('system');
		expect(parseTheme(undefined)).toBe('system');
		expect(parseTheme('dark.violet')).toBe('system');
		expect(parseTheme('<script>')).toBe('system');
	});

	it('forces a theme only when the person picked one; system follows the device with no script', () => {
		expect(themeAttribute('light')).toBe(' data-theme="light"');
		expect(themeAttribute('dark')).toBe(' data-theme="dark"');
		expect(themeAttribute('system')).toBe('');
	});

	it('puts the theme on <html> before the first frame', () => {
		const html = '<html lang="en"%nomi.theme%><head></head></html>';
		expect(withTheme(html, 'dark')).toBe('<html lang="en" data-theme="dark"><head></head></html>');
		expect(withTheme(html, 'system')).toBe('<html lang="en"><head></head></html>');
	});
});
