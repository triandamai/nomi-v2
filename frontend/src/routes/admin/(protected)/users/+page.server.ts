import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { CUSTOM_PERMISSION_RESOURCE } from '$lib/permissions';
import type { AdminUserDetail, AdminUserListResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

const PAGE_SIZE = 20;

export const load: PageServerLoad = async ({ url, cookies, fetch }) => {
	const query = url.searchParams.get('query') ?? '';
	const page = Number(url.searchParams.get('page') ?? '1') || 1;

	const params = new URLSearchParams({ page: String(page), page_size: String(PAGE_SIZE) });
	if (query) params.set('query', query);

	const usersResponse = await apiFetch(fetch, cookies, `/api/admin/users?${params}`);
	const result: AdminUserListResponse = usersResponse.ok
		? ((await usersResponse.json()) as AdminUserListResponse)
		: { users: [], total: 0 };

	return { users: result.users, total: result.total, page, pageSize: PAGE_SIZE, query };
};

function requireUserId(data: FormData): string | null {
	const userId = data.get('userId');
	return typeof userId === 'string' && userId ? userId : null;
}

export const actions: Actions = {
	promote: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		if (!userId) {
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

	loadUserDetail: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		if (!userId) {
			return fail(400, { error: 'Invalid user.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}`);
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to load user.' });
		}
		const user: AdminUserDetail = await response.json();
		return { success: true, user };
	},

	updateUser: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		if (!userId) {
			return fail(400, { error: 'Invalid user.' });
		}
		const displayName = data.get('display_name');
		const username = data.get('username');

		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/profile`, {
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
		const user: AdminUserDetail = await response.json();
		return { success: true, user };
	},

	grantPermission: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		const resourceField = data.get('resource');
		const customResource = data.get('customResource');
		const actions = data.getAll('actions').filter((a): a is string => typeof a === 'string');

		if (!userId || typeof resourceField !== 'string' || !resourceField) {
			return fail(400, { error: 'Resource is required.' });
		}
		const resource =
			resourceField === CUSTOM_PERMISSION_RESOURCE ? (typeof customResource === 'string' ? customResource : '') : resourceField;
		if (!resource) {
			return fail(400, { error: 'Resource is required.' });
		}
		if (actions.length === 0) {
			return fail(400, { error: 'Select at least one action.' });
		}

		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions`, {
			method: 'POST',
			body: JSON.stringify({ scope_type: 'admin', org_id: null, resource, actions }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to grant permission.' });
		}
		const permissions = await response.json();
		return { success: true, permissions };
	},

	revokePermission: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		const permissionId = data.get('permissionId');
		if (!userId || typeof permissionId !== 'string') {
			return fail(400, { error: 'Invalid permission.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions/${permissionId}`, {
			method: 'DELETE',
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to revoke permission.' });
		}
		const permissions = await response.json();
		return { success: true, permissions };
	},
};
