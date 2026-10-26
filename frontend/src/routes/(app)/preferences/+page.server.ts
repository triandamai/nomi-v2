import { fail } from '@sveltejs/kit';
import { DEFAULT_THEME, isThemeName } from '$lib/appearance';
import { isPinnable } from '$lib/drawerPins';
import { parseRecentsForm } from '$lib/drawerRecents';
import { isHomeLayoutId } from '$lib/homeLayouts';
import { apiFetch } from '$lib/server/api';
import { getLocale } from '$lib/paraglide/runtime';
import type { Preferences } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/preferences');
	const preferences: Preferences = response.ok
		? ((await response.json()) as Preferences)
		: { theme: 'system', accent_color: DEFAULT_THEME, timezone: 'UTC', language: getLocale(), has_stored_timezone: false };
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

	updateHomeLayout: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const layout = data.get('home_layout');
		if (!isHomeLayoutId(layout)) {
			return fail(400, { error: m.err_save_preference() });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ home_layout: layout }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.err_save_preference() });
		}
		return { success: true };
	},

	updateBackdrop: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ animated_backdrop: data.get('animated_backdrop') === 'true' }),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.err_save_preference() });
		}
		return { success: true };
	},

	updateRecents: async ({ request, cookies, fetch }) => {
		const recents = parseRecentsForm(await request.formData());
		if (!recents) {
			return fail(400, { error: m.recents_save_failed() });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify(recents),
		});
		if (!response.ok) {
			return fail(response.status, { error: m.recents_save_failed() });
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
