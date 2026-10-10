// The person's light/dark mode and colour theme, kept in a cookie so the server can put them on
// <html> in the very first response (hooks.server.ts): a reload shows their theme from the first
// frame instead of the default one until the app starts. The app layout refreshes it from the
// saved preferences on every load, and the appearance picker on every change.

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

/** The attributes for <html>. */
export function appearanceAttributes(appearance: Appearance | null): string {
	return appearance ? `data-theme="${appearance.theme}" data-color="${appearance.color}"` : '';
}

/** In the browser: remember a pick at once, for the next load. */
export function rememberAppearance(theme: string, color: string): void {
	const value = formatAppearance(theme, color);
	if (!value) return;
	document.cookie = `${APPEARANCE_COOKIE}=${value}; path=/; max-age=${MAX_AGE}; samesite=lax`;
}

export const APPEARANCE_COOKIE_OPTIONS = { path: '/', maxAge: MAX_AGE, httpOnly: false, sameSite: 'lax' as const };
