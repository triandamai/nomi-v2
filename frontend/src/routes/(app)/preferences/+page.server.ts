import { fail } from '@sveltejs/kit';
import { isThemeName } from '$lib/appearance';
import { isPinnable } from '$lib/drawerPins';
import { apiFetch } from '$lib/server/api';
import { getLocale } from '$lib/paraglide/runtime';
import type { Preferences } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/preferences');
	const preferences: Preferences = response.ok
		? ((await response.json()) as Preferences)
		: { theme: 'system', accent_color: 'canopy', timezone: 'UTC', language: getLocale(), has_stored_timezone: false };
	return { preferences };
};

export const actions: Actions = {
	updateTheme: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const theme = data.get('theme');
		if (typeof theme !== 'string' || !['light', 'dark', 'system'].includes(theme)) {
			return fail(400, { error: m.err_invalid_theme() });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ theme }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.err_save_preference() });
		}
		return { success: true };
	},

	updateAccentColor: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const accentColor = data.get('accent_color');
		if (!isThemeName(accentColor)) {
			return fail(400, { error: m.err_pick_theme() });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ accent_color: accentColor }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.err_save_preference() });
		}
		return { success: true };
	},

	updateTimezone: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const timezone = data.get('timezone');
		if (typeof timezone !== 'string' || timezone.length === 0) {
			return fail(400, { error: m.err_invalid_timezone() });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ timezone }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.err_save_preference() });
		}
		return { success: true };
	},

	updateDrawer: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		let pins: unknown;
		try {
			pins = JSON.parse(String(data.get('pins') ?? ''));
		} catch {
			pins = null;
		}
		if (!Array.isArray(pins) || !pins.every(isPinnable)) {
			return fail(400, { error: m.drawer_save_failed() });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ drawer_pins: pins }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.drawer_save_failed() });
		}
		return { success: true };
	},
};
