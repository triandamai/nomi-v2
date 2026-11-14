import { isRedirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { getLocale, isLocale } from '$lib/paraglide/runtime';
import { THEME_COOKIE, parseTheme, type Theme } from '$lib/theme';
import { personName } from '$lib/adapters/shell';
import type { Preferences, Profile } from '$lib/adapters/api';
import { adminAccess, type Access } from '$lib/admin/access';
import { apiFetch, type ApiEvent } from './api';
import { setLanguageCookie } from './locale';
import { setThemeCookie } from './theme';

/** What the admin console's shell needs: the person's access, who they are, how they see it. */
export type AdminShell = {
	access: Access;
	person: { name: string; avatar: string | null };
	theme: Theme;
	/** The person's saved time zone; null when they never saved one (the browser's decides). */
	timeZone: string | null;
};

async function json<T>(response: Response | null): Promise<T | null> {
	if (!response?.ok) return null;
	return (await response.json().catch(() => null)) as T | null;
}

/**
 * The console's access comes from the person's token (`GET /api/whoami`), which is what the
 * backend checks too: grants changed since they signed in show once the token is renewed.
 */
export async function loadAdminShell(event: ApiEvent): Promise<AdminShell> {
	const get = (path: string) =>
		apiFetch(event, path).catch((error) => {
			if (isRedirect(error)) throw error;
			return null;
		});
	const [claims, profile, preferences] = await Promise.all([
		get('/api/whoami').then(json<{ permissions?: string[] }>),
		get('/api/profile').then(json<Profile>),
		get('/api/preferences').then(json<Preferences>),
	]);

	// The saved language and theme win on every device, as in the app.
	if (preferences && isLocale(preferences.language) && preferences.language !== getLocale()) setLanguageCookie(event.cookies, preferences.language);
	const theme = parseTheme(preferences?.theme ?? event.cookies.get(THEME_COOKIE));
	if (preferences && event.cookies.get(THEME_COOKIE) !== theme) setThemeCookie(event.cookies, theme);

	return {
		access: adminAccess(claims?.permissions ?? []),
		person: { name: profile ? personName(profile) : m.shell_you(), avatar: profile?.avatar_url ?? null },
		theme,
		timeZone: preferences?.has_stored_timezone ? preferences.timezone : null,
	};
}
