import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminUserDetail } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}`);
	if (!response.ok) {
		throw error(response.status === 404 ? 404 : 500, 'Failed to load user.');
	}
	const user: AdminUserDetail = await response.json();
	return { user };
};

export const actions: Actions = {
	updateUser: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const displayName = data.get('display_name');
		const username = data.get('username');

		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/profile`, {
			method: 'PUT',
			body: JSON.stringify({
				display_name: typeof displayName === 'string' ? displayName : null,
				username: typeof username === 'string' ? username : null,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to update user.' });
		}
		return { success: true };
	},
};
