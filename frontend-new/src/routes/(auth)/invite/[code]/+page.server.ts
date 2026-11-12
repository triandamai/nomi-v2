import { fail, redirect } from '@sveltejs/kit';
import { displayEmail } from '$lib/auth/format';
import type { InviteProblem } from '$lib/auth/problems';
import { apiFetch } from '$lib/server/api';
import { readApiError } from '$lib/server/apiError';
import type { Actions, PageServerLoad } from './$types';

/** What `GET /api/invites/:code` says about an invite, for the person looking at it. */
export type InviteInfo = {
	org_id: string;
	org_name: string;
	inviter: string;
	role: 'owner' | 'admin' | 'member';
	state: 'open' | 'expired' | 'used' | 'member' | 'other_address';
	email: string | null;
	project?: { id: string; name: string; description: string | null; role: 'build' | 'prompt' | 'view' };
	expires_at: string;
};

const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

// An invite to a workspace, or to build one of its projects. Only someone signed in can see what
// it's for (the API asks who's looking), so a visitor is asked to make an account or sign in first,
// and comes back here after.

export const load: PageServerLoad = async (event) => {
	const code = event.params.code;
	const signedOut = { code, signedIn: false, invite: null, missing: false, failed: false };
	if (!event.locals.accessToken) return signedOut;
	const response = await apiFetch(event, `/api/invites/${encodeURIComponent(code)}`, undefined, { signedOut: 'return' }).catch(() => null);
	if (response?.status === 401) return signedOut;
	if (response?.status === 404) return { ...signedOut, signedIn: true, missing: true };
	if (!response?.ok) return { ...signedOut, signedIn: true, failed: true };
	const invite = (await response.json()) as InviteInfo;
	return { ...signedOut, signedIn: true, invite: { ...invite, email: invite.email && displayEmail(invite.email) } };
};

export const actions: Actions = {
	accept: async (event) => {
		const project = String((await event.request.formData()).get('project') ?? '');
		const response = await apiFetch(event, `/api/invites/${encodeURIComponent(event.params.code)}/accept`, { method: 'POST' });
		if (response.ok) {
			// An invite to build a project opens it; joining a workspace goes home.
			redirect(303, UUID.test(project) ? `/projects/${project}` : '/');
		}
		const error = await readApiError(response);
		const problem: InviteProblem =
			response.status === 410 ? (error.message.includes('expired') ? 'expired' : 'used') : response.status === 403 ? 'other_address' : 'failed';
		return fail(response.status, { problem });
	},
};
