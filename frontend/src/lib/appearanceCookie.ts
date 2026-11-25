// The person's light/dark mode and colour theme, kept in a cookie so the server can put them on
// <html> in the very first response (hooks.server.ts): a reload shows their theme from the first
// frame instead of the default one until the app starts. The app layout refreshes it from the
// saved preferences on every load, and the appearance picker on every change.

import { DEFAULT_THEME, THEMES } from '$lib/appearance';

export const APPEARANCE_COOKIE = 'nomi_appearance';
const MAX_AGE = 60 * 60 * 24 * 365;
const SAFE = /^[a-z-]{1,32}$/;

export interface Appearance {
	theme: string;
	color: string;
}

/** "theme.color", or null when it isn't one we wrote. */
export function parseAppearance(value: string | undefined | null): Appearance | null {
	if (!value) return null;
	const [theme, color] = value.split('.');
	return theme && color && SAFE.test(theme) && SAFE.test(color) ? { theme, color } : null;
}

export function formatAppearance(theme: string, color: string): string | null {
	return SAFE.test(theme) && SAFE.test(color) ? `${theme}.${color}` : null;
}

/** The attributes for <html>; with nothing saved, the default theme (Lagoon) and the system mode. */
export function appearanceAttributes(appearance: Appearance | null): string {
	return appearance ? `data-theme="${appearance.theme}" data-color="${appearance.color}"` : `data-color="${DEFAULT_THEME}"`;
}

function surfacePalette(color: string | undefined) {
	const found = THEMES.find((t) => t.value === color) ?? THEMES.find((t) => t.value === DEFAULT_THEME) ?? THEMES[0];
	return { light: found.light.surface, dark: found.dark.surface };
}

/**
 * The `<meta name="theme-color">` tag(s) for `<head>`. System follows the device with media
 * queries; an explicit light/dark pick drops the media query so Safari colours its bar to match.
 */
export function themeColorMeta(appearance: Appearance | null): string {
	const palette = surfacePalette(appearance?.color);
	if (appearance?.theme === 'light') return `<meta name="theme-color" content="${palette.light}" />`;
	if (appearance?.theme === 'dark') return `<meta name="theme-color" content="${palette.dark}" />`;
	return `<meta name="theme-color" content="${palette.light}" media="(prefers-color-scheme: light)" />\n\t\t<meta name="theme-color" content="${palette.dark}" media="(prefers-color-scheme: dark)" />`;
}

/**
 * Replaces `<meta name="theme-color">` in `<head>` with fresh nodes matching `theme` and `color`.
 * Replacing rather than mutating in place ensures Safari repaints its toolbar immediately.
 */
export function syncThemeColor(theme: string, color: string): void {
	const palette = surfacePalette(color);
	for (const tag of document.head.querySelectorAll('meta[name="theme-color"]')) tag.remove();
	const entries: { content: string; media?: string }[] =
		theme === 'light'
			? [{ content: palette.light }]
			: theme === 'dark'
				? [{ content: palette.dark }]
				: [
						{ content: palette.light, media: '(prefers-color-scheme: light)' },
						{ content: palette.dark, media: '(prefers-color-scheme: dark)' },
					];
	for (const entry of entries) {
		const meta = document.createElement('meta');
		meta.name = 'theme-color';
		meta.content = entry.content;
		if (entry.media) meta.media = entry.media;
		document.head.append(meta);
	}
}

/** In the browser: remember a pick at once, for the next load. */
export function rememberAppearance(theme: string, color: string): void {
	const value = formatAppearance(theme, color);
	if (!value) return;
	document.cookie = `${APPEARANCE_COOKIE}=${value}; path=/; max-age=${MAX_AGE}; samesite=lax`;
}

export const APPEARANCE_COOKIE_OPTIONS = { path: '/', maxAge: MAX_AGE, httpOnly: false, sameSite: 'lax' as const };
