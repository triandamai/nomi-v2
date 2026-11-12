import type { Cookies } from '@sveltejs/kit';
import { THEME_COOKIE, type Theme } from '$lib/theme';

/** A year: the saved theme is set again on every app load, so it never runs out in use. */
const THEME_MAX_AGE = 365 * 24 * 60 * 60;

/** Remembers the person's theme on this device, so the server renders the first frame in it. */
export function setThemeCookie(cookies: Cookies, theme: Theme): void {
	cookies.set(THEME_COOKIE, theme, { path: '/', sameSite: 'lax', httpOnly: true, maxAge: THEME_MAX_AGE });
}
