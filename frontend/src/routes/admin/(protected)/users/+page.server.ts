import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { CUSTOM_PERMISSION_RESOURCE } from '$lib/permissions';
import type { AdminSubscription, AdminUserDetail, AdminUserListResponse, Plan } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

const PAGE_SIZES = [10, 20, 50, 100];
const DEFAULT_PAGE_SIZE = 20;

export const load: PageServerLoad = async ({ url, cookies, fetch }) => {
	const query = url.searchParams.get('query') ?? '';
	const page = Math.max(1, Number(url.searchParams.get('page') ?? '1') || 1);
	const askedSize = Number(url.searchParams.get('size'));
	const pageSize = PAGE_SIZES.includes(askedSize) ? askedSize : DEFAULT_PAGE_SIZE;

	const params = new URLSearchParams({ page: String(page), page_size: String(pageSize) });
	if (query) params.set('query', query);

	const [usersResponse, plansResponse] = await Promise.all([
		apiFetch(fetch, cookies, `/api/admin/users?${params}`),
		apiFetch(fetch, cookies, '/api/admin/plans'),
	]);
	const result: AdminUserListResponse = usersResponse.ok
		? ((await usersResponse.json()) as AdminUserListResponse)
		: { users: [], total: 0 };
	const plans: Plan[] = plansResponse.ok ? ((await plansResponse.json()) as Plan[]) : [];

	return { users: result.users, total: result.total, page, pageSize, pageSizes: PAGE_SIZES, query, plans };
};

function requireUserId(data: FormData): string | null {
	const userId = data.get('userId');
	return typeof userId === 'string' && userId ? userId : null;
}

export const actions: Actions = {
	loadSubscription: async ({ request, cookies, fetch }) => {
		const userId = requireUserId(await request.formData());
		if (!userId) return fail(400, { error: m.err_invalid_user() });
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/subscription`);
		if (!response.ok) return fail(response.status, { error: (await response.text()) || m.err_load_user() });
		return { subscription: (await response.json()) as AdminSubscription };
	},

	/** Moves someone to a plan and/or overrides its allowance; they're notified (in the app and by email). */
	saveSubscription: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		const planId = data.get('plan_id');
		if (!userId || typeof planId !== 'string' || !planId) return fail(400, { error: m.err_invalid_user() });
		const quotaText = String(data.get('quota_override') ?? '').trim();
		const untilText = String(data.get('override_until') ?? '').trim();
		const quota = quotaText === '' ? null : Number(quotaText);
		if (quota !== null && !(Number.isInteger(quota) && quota >= 0)) return fail(400, { error: m.sub_quota_invalid() });
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/subscription`, {
			method: 'PUT',
			body: JSON.stringify({
				plan_id: planId,
				quota_override: quota,
				override_until: quota !== null && untilText ? new Date(`${untilText}T23:59:59`).toISOString() : null,
				note: String(data.get('note') ?? '').trim() || null,
			}),
		});
		if (!response.ok) return fail(response.status, { error: (await response.text()) || m.sub_save_failed() });
		return { subscription: (await response.json()) as AdminSubscription, saved: true };
	},

	promote: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		if (!userId) {
			return fail(400, { error: m.err_invalid_user() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions`, {
			method: 'POST',
			body: JSON.stringify({ scope_type: 'admin', org_id: null, resource: 'user', actions: ['view'] }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_promote_user() });
		}
		return { success: true };
	},

	loadUserDetail: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		if (!userId) {
			return fail(400, { error: m.err_invalid_user() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}`);
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_load_user() });
		}
		const user: AdminUserDetail = await response.json();
		return { success: true, user };
	},

	updateUser: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		if (!userId) {
			return fail(400, { error: m.err_invalid_user() });
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
			return fail(response.status, { error: message || m.err_update_user() });
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
			return fail(400, { error: m.err_resource_required() });
		}
		const resource =
			resourceField === CUSTOM_PERMISSION_RESOURCE ? (typeof customResource === 'string' ? customResource : '') : resourceField;
		if (!resource) {
			return fail(400, { error: m.err_resource_required() });
		}
		if (actions.length === 0) {
			return fail(400, { error: m.err_select_action() });
		}

		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions`, {
			method: 'POST',
			body: JSON.stringify({ scope_type: 'admin', org_id: null, resource, actions }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_grant_permission() });
		}
		const permissions = await response.json();
		return { success: true, permissions };
	},

	revokePermission: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const userId = requireUserId(data);
		const permissionId = data.get('permissionId');
		if (!userId || typeof permissionId !== 'string') {
			return fail(400, { error: m.err_invalid_permission() });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${userId}/permissions/${permissionId}`, {
			method: 'DELETE',
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || m.err_revoke_permission() });
		}
		const permissions = await response.json();
		return { success: true, permissions };
	},
};
