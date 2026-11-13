import { redirect } from '@sveltejs/kit';
import { signOut, signOutTarget } from '$lib/server/account';
import type { RequestHandler } from './$types';

/**
 * Logs this device out: the refresh token is revoked on the backend, the session cookies are
 * cleared, and the person lands on sign in (or a page on this site given as `redirect_to`, in the
 * query or the form). POST only, so another site can't sign someone out with a link.
 */
export const POST: RequestHandler = async (event) => {
	await signOut(event);
	const form = await event.request.formData().catch(() => null);
	const asked = event.url.searchParams.get('redirect_to') ?? form?.get('redirect_to');
	redirect(303, signOutTarget(typeof asked === 'string' ? asked : null));
};
