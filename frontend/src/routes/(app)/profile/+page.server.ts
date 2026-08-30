import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Profile } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

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
			return fail(response.status, { error: message || 'Failed to save profile.' });
		}
		return { success: true };
	},

	requestAvatarUploadUrl: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const contentType = data.get('content_type');
		if (typeof contentType !== 'string' || !contentType) {
			return fail(400, { error: 'Missing content type.' });
		}
		const response = await apiFetch(fetch, cookies, '/api/profile/avatar/upload-url', {
			method: 'POST',
			body: JSON.stringify({ content_type: contentType }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to prepare avatar upload.' });
		}
		const result = (await response.json()) as { upload_url: string; public_url: string };
		return { uploadUrl: result.upload_url, publicUrl: result.public_url };
	},
};
