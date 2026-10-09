import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Profile } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/profile');
	const profile: Profile | null = response.ok ? ((await response.json()) as Profile) : null;
	return { profile };
};

export const actions: Actions = {
	updateProfile: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const displayName = data.get('display_name');
		const username = data.get('username');
		const avatarUrl = data.get('avatar_url');

		const response = await apiFetch(fetch, cookies, '/api/profile', {
			method: 'PUT',
			body: JSON.stringify({
				display_name: typeof displayName === 'string' ? displayName : null,
				username: typeof username === 'string' ? username : null,
				avatar_url: typeof avatarUrl === 'string' && avatarUrl.length > 0 ? avatarUrl : null,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_save_profile() });
		}
		return { success: true };
	},
};
