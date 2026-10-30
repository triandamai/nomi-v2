// The person's theme: light, dark or system (follow the device). It's saved with their
// preferences and mirrored in a cookie so the server can put it on <html> before the first frame.
// The old app's accent colours are gone: the design has one accent.

export type Theme = 'light' | 'dark' | 'system';

/** Its own name, so the old app's `nomi_appearance` cookie (theme.colour) never leaks in. */
export const THEME_COOKIE = 'nm_theme';

export function parseTheme(value: string | undefined): Theme {
	return value === 'light' || value === 'dark' ? value : 'system';
}

/** The attribute for <html>. System needs none: the tokens follow the device by themselves. */
export function themeAttribute(theme: Theme): string {
	return theme === 'system' ? '' : ` data-theme="${theme}"`;
}

export function withTheme(html: string, theme: Theme): string {
	return html.replace('%nomi.theme%', themeAttribute(theme));
}
