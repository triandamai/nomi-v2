import { error, fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { CUSTOM_PERMISSION_RESOURCE } from '$lib/permissions';
import type { AdminUserDetail, OrgOption } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params, cookies, fetch }) => {
	const [detailResponse, orgsResponse] = await Promise.all([
		apiFetch(fetch, cookies, `/api/admin/users/${params.id}`),
		apiFetch(fetch, cookies, '/api/admin/orgs'),
	]);

	if (!detailResponse.ok) {
		throw error(detailResponse.status === 404 ? 404 : 500, 'Failed to load user.');
	}

	const user: AdminUserDetail = await detailResponse.json();
	const orgs: OrgOption[] = orgsResponse.ok ? await orgsResponse.json() : [];

	return { user, orgs };
};

export const actions: Actions = {
	grantPermission: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const scopeType = data.get('scopeType');
		const orgId = data.get('orgId');
		const resourceField = data.get('resource');
		const customResource = data.get('customResource');
		const actions = data.getAll('actions').filter((a): a is string => typeof a === 'string');

		if (typeof scopeType !== 'string' || typeof resourceField !== 'string' || !resourceField) {
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

		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/permissions`, {
			method: 'POST',
			body: JSON.stringify({
				scope_type: scopeType,
				org_id: scopeType === 'org' && typeof orgId === 'string' && orgId ? orgId : null,
				resource,
				actions,
			}),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to grant permission.' });
		}
		return { success: true };
	},

	revokePermission: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const permissionId = data.get('permissionId');
		if (typeof permissionId !== 'string') {
			return fail(400, { error: 'Invalid permission.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/permissions/${permissionId}`, {
			method: 'DELETE',
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to revoke permission.' });
		}
		return { success: true };
	},

	assignOrg: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const orgId = data.get('orgId');
		const role = data.get('role');
		if (typeof orgId !== 'string' || !orgId || typeof role !== 'string' || !role) {
			return fail(400, { error: 'Organization and role are required.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/memberships`, {
			method: 'POST',
			body: JSON.stringify({ org_id: orgId, role }),
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to assign organization.' });
		}
		return { success: true };
	},

	removeOrg: async ({ request, params, cookies, fetch }) => {
		const data = await request.formData();
		const orgId = data.get('orgId');
		if (typeof orgId !== 'string') {
			return fail(400, { error: 'Invalid organization.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/admin/users/${params.id}/memberships/${orgId}`, {
			method: 'DELETE',
		});
		if (!response.ok) {
			const message = await response.text();
			return fail(response.status, { error: message || 'Failed to remove organization.' });
		}
		return { success: true };
	},
};
