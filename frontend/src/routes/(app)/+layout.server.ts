import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { CrewRosterMember } from '$lib/crew';
import type { Preferences, Profile } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login');
	}

	const userEmail = cookies.get('user_email') ?? '';

	const [profileResponse, preferencesResponse, crewResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/profile'),
		apiFetch(fetch, cookies, '/api/preferences'),
		apiFetch(fetch, cookies, '/api/agents'),
	]);
	const crew: CrewRosterMember[] = crewResponse.ok ? ((await crewResponse.json()) as { agents: CrewRosterMember[] }).agents : [];
	const profile: Profile = profileResponse.ok
		? ((await profileResponse.json()) as Profile)
		: { display_name: null, username: null, email: userEmail, avatar_url: null };
	const preferences: Preferences = preferencesResponse.ok
		? ((await preferencesResponse.json()) as Preferences)
		: { theme: 'system', accent_color: 'green', timezone: 'UTC', has_stored_timezone: false };

	return { userEmail, profile, preferences, crew };
};
