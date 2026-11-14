import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { activeOrgId, switchActiveOrg } from '$lib/server/activeOrg';
import type { OrgList } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/orgs');
	if (!response.ok) throw error(response.status, m.org_err_action());
	const list = (await response.json()) as OrgList;
	return { orgs: list.orgs, projectOrgId: list.project_org_id, activeOrgId: activeOrgId(cookies.get('access_token')) };
};

export const actions: Actions = {
	/** Makes a workspace the active one: the new access token carries it. */
	switch: async ({ request, cookies, fetch }) => {
		const orgId = String((await request.formData()).get('org_id') ?? '');
		if (!orgId) return fail(400, { switchError: m.org_switch_failed() });
		if (!(await switchActiveOrg(fetch, cookies, orgId))) return fail(400, { switchError: m.org_switch_failed() });
		return { switched: orgId };
	},
	create: async ({ request, cookies, fetch }) => {
		const name = String((await request.formData()).get('name') ?? '').trim();
		if (!name) return fail(400, { error: m.org_err_name() });
		const response = await apiFetch(fetch, cookies, '/api/orgs', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name }),
		});
		if (!response.ok) return fail(response.status, { error: m.org_err_create() });
		const { id } = (await response.json()) as { id: string };
		throw redirect(303, `/organizations/${id}`);
	},
};
