import { DEFAULT_THEME } from '$lib/appearance';
import { APPEARANCE_COOKIE, APPEARANCE_COOKIE_OPTIONS, formatAppearance } from '$lib/appearanceCookie';
import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { getLocale } from '$lib/paraglide/runtime';
import { setLanguageCookie } from '$lib/server/locale';
import type { Preferences } from '$lib/types';
import { ADMIN_AREAS, adminAccess, areaForPath, can } from '$lib/permissions';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch, url }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const response = await apiFetch(fetch, cookies, '/api/whoami');
	if (!response.ok) {
		throw redirect(303, '/login?redirect_to=/admin');
	}

	const claims = (await response.json()) as { permissions: string[] };
	const access = adminAccess(claims.permissions);
	const first = ADMIN_AREAS.find((area) => can(access, area.resource));
	if (!first) {
		throw redirect(303, '/?error=forbidden');
	}
	// A page of an area they can't see: the first area they can.
	const area = areaForPath(url.pathname);
	if (area && !can(access, area.resource)) {
		const home = first.resource === 'dashboard' ? '/admin' : (first.paths[0] ?? '/admin');
		if (home !== url.pathname) throw redirect(303, home);
	}

	const preferencesResponse = await apiFetch(fetch, cookies, '/api/preferences');
	const preferences: Preferences = preferencesResponse.ok
		? ((await preferencesResponse.json()) as Preferences)
		: { theme: 'system', accent_color: DEFAULT_THEME, timezone: 'UTC', language: getLocale(), has_stored_timezone: false };

	// The saved language wins on every device; the cookie makes the next render use it.
	if (preferencesResponse.ok && preferences.language !== getLocale()) setLanguageCookie(cookies, preferences.language);
	// Saved appearance on <html> from the first frame of the next load (see hooks.server.ts).
	const appearance = preferencesResponse.ok ? formatAppearance(preferences.theme, preferences.accent_color) : null;
	if (appearance && cookies.get(APPEARANCE_COOKIE) !== appearance) cookies.set(APPEARANCE_COOKIE, appearance, APPEARANCE_COOKIE_OPTIONS);

	return { access, preferences };
};
