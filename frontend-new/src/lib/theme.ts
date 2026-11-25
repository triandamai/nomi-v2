// The person's theme: light, dark or system (follow the device). It's saved with their
// preferences and mirrored in a cookie so the server can put it on <html> before the first frame.
// The old app's accent colours are gone: the design has one accent.

import { THEME_COLOR } from '$lib/seo/seo';

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

/**
 * The `<meta name="theme-color">` tag(s) for `<head>`. System follows the device with media
 * queries; an explicit pick drops the media query so Safari colours its bar to match the app.
 */
export function themeColorMeta(theme: Theme): string {
	if (theme === 'system') {
		return `<meta name="theme-color" content="${THEME_COLOR.light}" media="(prefers-color-scheme: light)" />\n\t\t<meta name="theme-color" content="${THEME_COLOR.dark}" media="(prefers-color-scheme: dark)" />`;
	}
	return `<meta name="theme-color" content="${THEME_COLOR[theme]}" />`;
}

export function withTheme(html: string, theme: Theme): string {
	return html.replace('%nomi.theme%', themeAttribute(theme)).replace('%nomi.theme-color%', themeColorMeta(theme));
}

/**
 * Replaces `<meta name="theme-color">` in `<head>` with fresh nodes matching `theme`.
 * Replacing rather than mutating in place ensures Safari repaints its toolbar immediately.
 */
export function syncThemeColor(theme: Theme): void {
	for (const tag of document.head.querySelectorAll('meta[name="theme-color"]')) tag.remove();
	const entries: { content: string; media?: string }[] =
		theme === 'system'
			? [
					{ content: THEME_COLOR.light, media: '(prefers-color-scheme: light)' },
					{ content: THEME_COLOR.dark, media: '(prefers-color-scheme: dark)' },
				]
			: [{ content: THEME_COLOR[theme] }];
	for (const entry of entries) {
		const meta = document.createElement('meta');
		meta.name = 'theme-color';
		meta.content = entry.content;
		if (entry.media) meta.media = entry.media;
		document.head.append(meta);
	}
}

/** Applies the theme on `<html>` and updates `<meta name="theme-color">` to match. */
export function applyTheme(theme: Theme): void {
	if (theme === 'system') delete document.documentElement.dataset.theme;
	else document.documentElement.dataset.theme = theme;
	syncThemeColor(theme);
}
