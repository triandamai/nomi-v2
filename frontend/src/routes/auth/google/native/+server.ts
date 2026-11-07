import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import { markGoogleNative } from '$lib/server/googleNative';
import type { RequestHandler } from './$types';

/**
 * "Continue with Google" in the native app: the app opens this page in the browser, which goes on to Google's
 * sign-in; the callback then hands Google's answer back to the app (`nomi://auth/google`).
 */
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await fetch(apiUrl('/api/auth/google/start'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ invite_code: null }),
	}).catch(() => null);
	if (!response?.ok) redirect(303, 'nomi://auth/google?error=unavailable');
	const { url } = (await response.json()) as { url: string };
	markGoogleNative(cookies);
	redirect(303, url);
};
