import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { AdminUserListResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

const PAGE_SIZE = 20;

export const load: PageServerLoad = async ({ url, cookies, fetch }) => {
	const query = url.searchParams.get('query') ?? '';
	const page = Number(url.searchParams.get('page') ?? '1') || 1;

	const params = new URLSearchParams({ page: String(page), page_size: String(PAGE_SIZE) });
	if (query) params.set('query', query);

	const response = await apiFetch(fetch, cookies, `/api/admin/users?${params}`);
	const result: AdminUserListResponse = response.ok
		? ((await response.json()) as AdminUserListResponse)
		: { users: [], total: 0 };

	return { users: result.users, total: result.total, page, pageSize: PAGE_SIZE, query };
};

export const actions: Actions = {
	promote: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = data.get('userId');
		if (typeof userId !== 'string') {
			return fail(400, { error: 'Invalid user.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions`, {
			method: 'POST',
			body: JSON.stringify({ scope_type: 'admin', org_id: null, resource: 'user', actions: ['view'] }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to promote user.' });
		}
		return { success: true };
	},
};
