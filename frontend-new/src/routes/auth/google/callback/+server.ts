import { redirect } from '@sveltejs/kit';
import type { GoogleError } from '$lib/auth/problems';
import { withRedirect } from '$lib/auth/redirects';
import { callAuth, GOOGLE_RETURN_COOKIE, googleErrorFor, rememberMethod } from '$lib/server/authFlow';
import { completeSignIn, safeRedirect } from '$lib/server/signIn';
import type { RequestHandler } from './$types';

type Finished = { email: string; is_new: boolean; linked: boolean; access_token: string | null; refresh_token: string | null };

/**
 * Google sends people back here (the backend's `sign_in_redirect_uri`). Signed out, it signs them
 * in (a new account goes to its first run); signed in, it was "add Google" from Settings. Whatever
 * went wrong goes back as a known reason, never Google's or the backend's raw text.
 */
export const GET: RequestHandler = async (event) => {
	const { url, cookies, locals } = event;
	const returnTo = safeRedirect(cookies.get(GOOGLE_RETURN_COOKIE));
	cookies.delete(GOOGLE_RETURN_COOKIE, { path: '/' });
	const linking = Boolean(locals.accessToken);
	const back = (reason: GoogleError) => (linking ? `/settings?google_error=${reason}` : withRedirect('/sign-in', returnTo, { google_error: reason }));

	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');
	if (url.searchParams.get('error') || !code || !state) redirect(303, back('cancelled'));

	const response = await callAuth(event.fetch, '/api/auth/google/callback', { code, state });
	if (!response) redirect(303, back('failed'));
	if (!response.ok) redirect(303, back(await googleErrorFor(response)));

	const result = (await response.json()) as Finished;
	if (result.linked || !result.access_token || !result.refresh_token) redirect(303, '/settings?google=linked');
	await completeSignIn(event, { access_token: result.access_token, refresh_token: result.refresh_token }, result.is_new);
	rememberMethod(cookies, 'google');
	redirect(303, result.is_new ? withRedirect('/first-run', returnTo) : returnTo);
};
