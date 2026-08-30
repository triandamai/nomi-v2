import { apiFetch } from '$lib/server/api';
import type { Profile } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/profile');
	const profile: Profile | null = response.ok ? ((await response.json()) as Profile) : null;
	return { profile };
};
