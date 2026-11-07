import type { Cookies } from '@sveltejs/kit';

/** Marks a Google sign-in the native app started, so the callback hands Google's answer back to the app. */
export const GOOGLE_NATIVE_COOKIE = 'google_native';
/** Long enough to pick an account on Google's page. */
const GOOGLE_NATIVE_MAX_AGE_S = 10 * 60;
/** The app's own scheme; it finishes the sign-in with its SDK. */
export const GOOGLE_NATIVE_RETURN = 'nomi://auth/google';

export function markGoogleNative(cookies: Cookies) {
	cookies.set(GOOGLE_NATIVE_COOKIE, '1', { path: '/', httpOnly: true, sameSite: 'lax', maxAge: GOOGLE_NATIVE_MAX_AGE_S });
}

/**
 * Where the callback sends a sign-in the app started (and forgets that it did): the app's link with Google's answer,
 * or null for a sign-in on the web.
 */
export function nativeReturn(cookies: Cookies, params: URLSearchParams): string | null {
	if (!cookies.get(GOOGLE_NATIVE_COOKIE)) return null;
	cookies.delete(GOOGLE_NATIVE_COOKIE, { path: '/' });
	const answer = new URLSearchParams();
	for (const key of ['code', 'state', 'error']) {
		const value = params.get(key);
		if (value) answer.set(key, value);
	}
	return `${GOOGLE_NATIVE_RETURN}?${answer}`;
}
