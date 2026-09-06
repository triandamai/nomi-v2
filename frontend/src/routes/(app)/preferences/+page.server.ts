import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Preferences } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/preferences');
	const preferences: Preferences = response.ok ? ((await response.json()) as Preferences) : { theme: 'system', accent_color: 'green' };
	return { preferences };
};

export const actions: Actions = {
	updateTheme: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const theme = data.get('theme');
		if (typeof theme !== 'string' || !['light', 'dark', 'system'].includes(theme)) {
			return fail(400, { error: 'Invalid theme.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ theme }),
		});
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to save preference.' });
		}
		return { success: true };
	},

	updateAccentColor: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const accentColor = data.get('accent_color');
		if (
			typeof accentColor !== 'string' ||
			!['green', 'blue', 'purple', 'pink', 'orange', 'teal'].includes(accentColor)
		) {
			return fail(400, { error: 'Invalid color.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/preferences', {
			method: 'PUT',
			body: JSON.stringify({ accent_color: accentColor }),
		});
		if (!response.ok) {
			return fail(response.status, { error: 'Failed to save preference.' });
		}
		return { success: true };
	},
};
