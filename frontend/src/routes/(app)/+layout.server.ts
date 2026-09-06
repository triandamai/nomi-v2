import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Preferences, Profile } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login');
	}

	const userEmail = cookies.get('user_email') ?? '';

	const [profileResponse, preferencesResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/profile'),
		apiFetch(fetch, cookies, '/api/preferences'),
	]);
	const profile: Profile = profileResponse.ok
		? ((await profileResponse.json()) as Profile)
		: { display_name: null, username: null, email: userEmail, avatar_url: null };
	const preferences: Preferences = preferencesResponse.ok
		? ((await preferencesResponse.json()) as Preferences)
		: { theme: 'system', accent_color: 'green' };

	return { userEmail, profile, preferences };
};
