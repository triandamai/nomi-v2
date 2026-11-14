import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import { markGoogleNative } from '$lib/server/googleNative';
import type { RequestHandler } from './$types';

/**
 * "Continue with Google" in the native app: the app opens this page in the browser, which goes on to Google's
 * sign-in; the callback then hands Google's answer back to the app (`nomi://auth/google`).
 */
export const GET: RequestHandler = async ({ cookies, fetch, url }) => {
	const response = await fetch(apiUrl('/api/auth/google/start'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		// Back to this site's callback, which holds the native mark.
		body: JSON.stringify({ invite_code: null, origin: url.origin }),
	}).catch(() => null);
	if (!response?.ok) redirect(303, 'nomi://auth/google?error=unavailable');
	const { url: googleUrl } = (await response.json()) as { url: string };
	markGoogleNative(cookies);
	redirect(303, googleUrl);
};
