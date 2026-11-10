import { isRedirect, redirect } from '@sveltejs/kit';
import { SIGN_IN_PATH } from '$lib/server/api';
import { getMaintenance } from '$lib/server/maintenance';
import { loadShell } from '$lib/server/shell';
import type { LayoutServerLoad } from './$types';

/**
 * The signed-in app around every page: everyone waits on the maintenance page while Nomi is down,
 * a signed-out visitor signs in and comes back here, and everyone else gets the shell's data.
 * `invalidate('app:shell')` reloads it (a workspace picked, a chat that moved on elsewhere).
 */
export const load: LayoutServerLoad = async ({ locals, cookies, fetch, url, request, depends, untrack }) => {
	depends('app:shell');
	if (await getMaintenance(fetch)) redirect(307, '/maintenance');

	// Read without tracking: the shell doesn't reload on every navigation, only when invalidated.
	const here = untrack(() => url.pathname + url.search);
	const signIn = `${SIGN_IN_PATH}?redirect_to=${encodeURIComponent(here)}`;
	if (!locals.accessToken) redirect(303, signIn);

	try {
		return await loadShell({ fetch, cookies, locals, userAgent: request.headers.get('user-agent') ?? '' });
	} catch (error) {
		// The sign-in couldn't be renewed: sign in again and come back to this page.
		if (isRedirect(error) && error.location === SIGN_IN_PATH) redirect(303, signIn);
		throw error;
	}
};
