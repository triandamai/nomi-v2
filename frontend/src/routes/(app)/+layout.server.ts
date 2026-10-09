import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { getLocale } from '$lib/paraglide/runtime';
import { setLanguageCookie } from '$lib/server/locale';
import type { CrewRosterMember } from '$lib/crew';
import type { Preferences, Profile, UsageBrief } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login');
	}

	const userEmail = cookies.get('user_email') ?? '';

	const [profileResponse, preferencesResponse, crewResponse, usageResponse, unreadResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/profile'),
		apiFetch(fetch, cookies, '/api/preferences'),
		apiFetch(fetch, cookies, '/api/agents'),
		apiFetch(fetch, cookies, '/api/usage/brief'),
		apiFetch(fetch, cookies, '/api/notifications/unread'),
	]);
	const unread = unreadResponse.ok ? ((await unreadResponse.json()) as { unread: number }).unread : 0;
	// The drawer's usage meter; it just doesn't show when usage can't be read.
	const usage: UsageBrief | null = usageResponse.ok ? ((await usageResponse.json()) as UsageBrief) : null;
	const crew: CrewRosterMember[] = crewResponse.ok ? ((await crewResponse.json()) as { agents: CrewRosterMember[] }).agents : [];
	const profile: Profile = profileResponse.ok
		? ((await profileResponse.json()) as Profile)
		: { display_name: null, username: null, email: userEmail, avatar_url: null };
	const preferences: Preferences = preferencesResponse.ok
		? ((await preferencesResponse.json()) as Preferences)
		: { theme: 'system', accent_color: 'canopy', timezone: 'UTC', language: getLocale(), has_stored_timezone: false };

	// The saved language wins on every device; the cookie makes the next render use it.
	if (preferencesResponse.ok && preferences.language !== getLocale()) setLanguageCookie(cookies, preferences.language);

	return { userEmail, profile, preferences, crew, usage, unread };
};
