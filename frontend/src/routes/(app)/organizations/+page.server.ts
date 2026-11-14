import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { activeOrgId, switchActiveOrg } from '$lib/server/activeOrg';
import type { MyInvite, OrgList } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/orgs');
	if (!response.ok) throw error(response.status, m.org_err_action());
	const list = (await response.json()) as OrgList;
	// Pending invites are a bonus: the page still works without them.
	const invitesResponse = await apiFetch(fetch, cookies, '/api/invites');
	const invites = invitesResponse.ok ? ((await invitesResponse.json()) as { invites: MyInvite[] }).invites : [];
	return {
		orgs: list.orgs,
		projectOrgId: list.project_org_id,
		openOrgId: list.open_org_id ?? null,
		invites,
		activeOrgId: activeOrgId(cookies.get('access_token')),
	};
};

function post(fetch: typeof globalThis.fetch, cookies: Parameters<typeof apiFetch>[1], path: string, method: string, body?: unknown) {
	return apiFetch(fetch, cookies, path, {
		method,
		headers: { 'Content-Type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body),
	});
}

export const actions: Actions = {
	/** Makes a workspace the active one: the new access token carries it. */
	switch: async ({ request, cookies, fetch }) => {
		const orgId = String((await request.formData()).get('org_id') ?? '');
		if (!orgId) return fail(400, { switchError: m.org_switch_failed() });
		if (!(await switchActiveOrg(fetch, cookies, orgId))) return fail(400, { switchError: m.org_switch_failed() });
		return { switched: orgId };
	},
	/** Joins a workspace someone invited the person to, and opens it. */
	join: async ({ request, cookies, fetch }) => {
		const code = String((await request.formData()).get('code') ?? '');
		if (!code) return fail(400, { inviteError: m.orgs_invite_failed() });
		const response = await post(fetch, cookies, `/api/invites/${encodeURIComponent(code)}/accept`, 'POST');
		if (!response.ok) return fail(response.status, { inviteError: m.orgs_invite_failed() });
		const { org_id } = (await response.json()) as { org_id: string };
		throw redirect(303, `/organizations/${org_id}`);
	},
	decline: async ({ request, cookies, fetch }) => {
		const code = String((await request.formData()).get('code') ?? '');
		if (!code) return fail(400, { inviteError: m.orgs_invite_failed() });
		const response = await post(fetch, cookies, `/api/invites/${encodeURIComponent(code)}/decline`, 'POST');
		if (!response.ok) return fail(response.status, { inviteError: m.orgs_invite_failed() });
		return { declined: code };
	},
	/** Where Nomi opens on sign-in: a workspace, or the last used ("" here). */
	openIn: async ({ request, cookies, fetch }) => {
		const orgId = String((await request.formData()).get('org_id') ?? '');
		const response = await post(fetch, cookies, '/api/preferences/open-org', 'PUT', { org_id: orgId || null });
		if (!response.ok) return fail(response.status, { openError: m.orgs_open_failed() });
		return { openSaved: true };
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
