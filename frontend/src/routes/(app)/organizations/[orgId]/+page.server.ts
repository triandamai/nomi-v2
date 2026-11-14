import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { OrgDetail } from '$lib/types';
import { deleteRefusal } from '$lib/orgs';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
	const response = await apiFetch(fetch, cookies, `/api/orgs/${params.orgId}`);
	if (response.status === 404) throw error(404, m.org_err_action());
	if (!response.ok) throw error(response.status, m.org_err_action());
	return { org: (await response.json()) as OrgDetail };
};

function send(fetch: typeof globalThis.fetch, cookies: Parameters<typeof apiFetch>[1], path: string, method: string, body?: unknown) {
	return apiFetch(fetch, cookies, path, {
		method,
		headers: { 'Content-Type': 'application/json' },
		body: body === undefined ? undefined : JSON.stringify(body),
	});
}

/** The message to show when a change to the team is refused. */
function failure(status: number, fallback: string) {
	if (status === 409) return fail(409, { error: m.org_err_last_owner() });
	return fail(status, { error: fallback });
}

export const actions: Actions = {
	rename: async ({ request, cookies, fetch, params }) => {
		const name = String((await request.formData()).get('name') ?? '').trim();
		if (!name) return fail(400, { error: m.org_err_name() });
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}`, 'PATCH', { name });
		if (!response.ok) return fail(response.status, { error: m.org_err_action() });
		return { renamed: true };
	},

	invite: async ({ request, cookies, fetch, params }) => {
		const data = await request.formData();
		const email = String(data.get('email') ?? '').trim();
		const role = String(data.get('role') ?? 'member');
		if (!email.includes('@')) return fail(400, { inviteError: m.org_err_invite_email(), email });
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/invites`, 'POST', { email, role });
		if (response.status === 409) return fail(409, { inviteError: m.org_err_already_member(), email });
		if (response.status === 400) return fail(400, { inviteError: m.org_err_invite_email(), email });
		if (!response.ok) return fail(response.status, { inviteError: m.org_err_invite(), email });
		return { invited: email.toLowerCase() };
	},

	revoke: async ({ request, cookies, fetch, params }) => {
		const code = String((await request.formData()).get('code') ?? '');
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/invites/${encodeURIComponent(code)}`, 'DELETE');
		if (!response.ok) return failure(response.status, m.org_err_action());
		return { revoked: true };
	},

	role: async ({ request, cookies, fetch, params }) => {
		const data = await request.formData();
		const userId = String(data.get('user_id') ?? '');
		const role = String(data.get('role') ?? '');
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/members/${userId}`, 'PATCH', { role });
		if (!response.ok) return failure(response.status, m.org_err_action());
		return { changed: true };
	},

	remove: async ({ request, cookies, fetch, params }) => {
		const userId = String((await request.formData()).get('user_id') ?? '');
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/members/${userId}`, 'DELETE');
		if (!response.ok) return failure(response.status, m.org_err_action());
		return { removed: true };
	},

	resend: async ({ request, cookies, fetch, params }) => {
		const code = String((await request.formData()).get('code') ?? '');
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/invites/${encodeURIComponent(code)}/resend`, 'POST');
		if (!response.ok) return fail(response.status, { error: m.org_err_action() });
		const { email } = (await response.json()) as { email: string | null };
		return { resent: email ?? '' };
	},

	/** A link anyone can join with, as `role`; the page copies it. */
	link: async ({ request, cookies, fetch, params }) => {
		const role = String((await request.formData()).get('role') ?? 'member');
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/invite-links`, 'POST', { role });
		if (!response.ok) return fail(response.status, { error: m.org_err_action() });
		const { code } = (await response.json()) as { code: string };
		return { linkCode: code };
	},

	delete: async ({ cookies, fetch, params }) => {
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}`, 'DELETE');
		if (response.status === 409) return fail(409, { error: deleteRefusal(await response.text()) });
		if (response.status === 403) return fail(403, { error: m.org_delete_only_owners() });
		if (!response.ok) return fail(response.status, { error: m.org_err_action() });
		throw redirect(303, '/organizations');
	},

	leave: async ({ cookies, fetch, params }) => {
		const detail = await apiFetch(fetch, cookies, `/api/orgs/${params.orgId}`);
		if (!detail.ok) return fail(detail.status, { error: m.org_err_action() });
		const { me } = (await detail.json()) as OrgDetail;
		const response = await send(fetch, cookies, `/api/orgs/${params.orgId}/members/${me}`, 'DELETE');
		if (!response.ok) return failure(response.status, m.org_err_action());
		throw redirect(303, '/organizations');
	},
};
