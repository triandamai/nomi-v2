import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { InviteInfo } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
	const response = await apiFetch(fetch, cookies, `/api/invites/${encodeURIComponent(params.code)}`);
	if (response.status === 404) return { invite: null };
	if (!response.ok) throw error(response.status, m.invite_err_accept());
	return { invite: (await response.json()) as InviteInfo };
};

export const actions: Actions = {
	accept: async ({ cookies, fetch, params }) => {
		const response = await apiFetch(fetch, cookies, `/api/invites/${encodeURIComponent(params.code)}/accept`, { method: 'POST' });
		if (!response.ok) return fail(response.status, { error: m.invite_err_accept() });
		// An invite to build a project opens its chat; otherwise joining makes it the organization Projects shows.
		const accepted = (await response.json().catch(() => ({}))) as { project_session_id?: string };
		throw redirect(303, accepted.project_session_id ? `/projects/session/${accepted.project_session_id}` : '/projects');
	},
};
