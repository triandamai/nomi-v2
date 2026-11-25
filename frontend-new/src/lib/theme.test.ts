import { describe, expect, it } from 'vitest';
import { THEME_COLOR } from '$lib/seo/seo';
import { THEME_COOKIE, parseTheme, themeAttribute, themeColorMeta, withTheme } from './theme';

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

	it('renders theme-color meta tags matching the chosen theme so Safari colours its bar', () => {
		expect(themeColorMeta('light')).toBe(`<meta name="theme-color" content="${THEME_COLOR.light}" />`);
		expect(themeColorMeta('dark')).toBe(`<meta name="theme-color" content="${THEME_COLOR.dark}" />`);
		expect(themeColorMeta('system')).toContain(`content="${THEME_COLOR.light}" media="(prefers-color-scheme: light)"`);
		expect(themeColorMeta('system')).toContain(`content="${THEME_COLOR.dark}" media="(prefers-color-scheme: dark)"`);
	});

	it('puts the theme and theme-color on the document before the first frame', () => {
		const html = '<html lang="en"%nomi.theme%><head>%nomi.theme-color%</head></html>';
		expect(withTheme(html, 'dark')).toBe(`<html lang="en" data-theme="dark"><head><meta name="theme-color" content="${THEME_COLOR.dark}" /></head></html>`);
		expect(withTheme(html, 'light')).toBe(`<html lang="en" data-theme="light"><head><meta name="theme-color" content="${THEME_COLOR.light}" /></head></html>`);
		expect(withTheme(html, 'system')).toContain('<html lang="en"><head><meta name="theme-color"');
	});
});
