import { redirect } from '@sveltejs/kit';
import { inviteCodeFrom, withRedirect } from '$lib/auth/redirects';
import { callAuth, GOOGLE_RETURN_COOKIE } from '$lib/server/authFlow';
import { safeRedirect } from '$lib/server/signIn';
import type { RequestHandler } from './$types';

/**
 * "Continue with Google" on sign-in and sign-up: off to Google's page. Where to go afterwards is
 * kept in a cookie while the person is at Google; an invite they were on travels with the request,
 * so a new account joins that workspace.
 */
export const POST: RequestHandler = async ({ request, fetch, cookies }) => {
	const redirectTo = safeRedirect(String((await request.formData()).get('redirect_to') ?? '') || null);
	const response = await callAuth(fetch, '/api/auth/google/start', { invite_code: inviteCodeFrom(redirectTo) });
	if (!response?.ok) redirect(303, withRedirect('/sign-in', redirectTo, { google_error: 'unavailable' }));
	const { url } = (await response.json()) as { url: string };
	cookies.set(GOOGLE_RETURN_COOKIE, redirectTo, { path: '/', httpOnly: true, sameSite: 'lax', maxAge: 600 });
	redirect(303, url);
};
