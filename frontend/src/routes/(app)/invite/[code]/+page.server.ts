import { error, fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { InviteInfo } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch, params }) => {
	const response = await apiFetch(fetch, cookies, `/api/invites/${encodeURIComponent(params.code)}`);
	if (response.status === 404) return { code: params.code, invite: null };
	if (!response.ok) throw error(response.status, m.invite_err_accept());
	return { code: params.code, invite: (await response.json()) as InviteInfo };
};

export const actions: Actions = {
	accept: async ({ cookies, fetch, params }) => {
		const response = await apiFetch(fetch, cookies, `/api/invites/${encodeURIComponent(params.code)}/accept`, { method: 'POST' });
		if (!response.ok) {
			const text = await response.text();
			const isOther = response.status === 403 || text.includes('another address');
			return fail(response.status, {
				error: isOther ? m.invite_failed_other_account({ email: '' }) : m.invite_err_accept(),
				isOther,
			});
		}
		// An invite to build a project opens its chat; otherwise joining makes it the organization Projects shows.
		const accepted = (await response.json().catch(() => ({}))) as { project_session_id?: string };
		throw redirect(303, accepted.project_session_id ? `/projects/session/${accepted.project_session_id}` : '/projects');
	},
};
