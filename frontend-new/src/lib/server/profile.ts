import { isRedirect } from '@sveltejs/kit';
import type { ProfileDetails } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import type { Outcome } from './chat';

// Settings › Profile's calls to the backend, as the person: their profile with its numbers, the
// projects that are live (only those can be featured: published at least once), saving what
// changed, and whether a username is free. Logic and API use from the old app
// (frontend/src/routes/(app)/profile).

export type ProfileChange = Partial<
	Pick<ProfileDetails, 'display_name' | 'username' | 'location' | 'show_stats_publicly' | 'show_crew_publicly' | 'showcase_project_ids'> & { avatar_url: string }
>;

/** A backend that can't be reached answers like one that's down. */
async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

export async function loadProfile(event: ApiEvent): Promise<{ profile: ProfileDetails; live: { id: string; name: string }[] } | null> {
	const [profile, projects] = await Promise.all([call(event, '/api/profile'), call(event, '/api/projects')]);
	if (!profile.ok) return null;
	const list = projects.ok ? ((await projects.json()) as { projects: { id: string; name: string; shipyard_app: string | null }[] }).projects : [];
	return {
		profile: (await profile.json()) as ProfileDetails,
		live: list.filter((project) => project.shipyard_app).map((project) => ({ id: project.id, name: project.name })),
	};
}

export async function saveProfile(event: ApiEvent, change: ProfileChange): Promise<Outcome<ProfileDetails>> {
	const response = await call(event, '/api/profile', { method: 'PUT', body: JSON.stringify(change) });
	if (!response.ok) return { ok: false, status: response.status, message: (await readApiError(response)).message };
	return { ok: true, data: (await response.json()) as ProfileDetails };
}

export async function checkUsername(event: ApiEvent, username: string): Promise<{ available: boolean; reason: string | null }> {
	const response = await call(event, `/api/profile/check-username?username=${encodeURIComponent(username)}`);
	if (!response.ok) return { available: false, reason: null };
	return (await response.json()) as { available: boolean; reason: string | null };
}
