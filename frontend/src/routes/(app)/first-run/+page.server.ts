import { fail, redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { Actions, PageServerLoad } from './$types';

/** Only an app path to go on to, never another site. */
function next(url: URL): string {
	const to = url.searchParams.get('next') ?? '/';
	return to.startsWith('/') && !to.startsWith('//') ? to : '/';
}

export const load: PageServerLoad = async ({ parent, url }) => {
	const { profile, userEmail } = await parent();
	// The name when we know it: what they set, or the address before the @.
	const name = profile?.display_name || (userEmail ?? '').split('@')[0] || '';
	return { name, next: next(url) };
};

export const actions: Actions = {
	// Saves the name (keeping the rest of the profile as it is) and goes on into the app.
	default: async ({ request, cookies, fetch, url }) => {
		const name = String((await request.formData()).get('name') ?? '').trim();
		if (!name) return fail(400, { error: m.firstrun_name() });
		const current = await apiFetch(fetch, cookies, '/api/profile');
		const profile = current.ok ? ((await current.json()) as { username: string | null; avatar_url: string | null }) : { username: null, avatar_url: null };
		const response = await apiFetch(fetch, cookies, '/api/profile', {
			method: 'PUT',
			body: JSON.stringify({ display_name: name, username: profile.username, avatar_url: profile.avatar_url }),
		});
		if (!response.ok) return fail(response.status, { error: m.err_save_profile() });
		throw redirect(303, next(url));
	},
};
