import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { PublicProfile } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const rawUsername = params.username.replace(/^@/, '');
	const response = await apiFetch(fetch, cookies, `/api/public/profile/${encodeURIComponent(rawUsername)}`);

	if (!response.ok) {
		throw error(response.status === 404 ? 404 : 500, response.status === 404 ? 'Profile not found' : 'Failed to load profile');
	}

	const profile = (await response.json()) as PublicProfile;
	return { profile };
};
