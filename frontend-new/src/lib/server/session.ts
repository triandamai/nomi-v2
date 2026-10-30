import type { Cookies } from '@sveltejs/kit';

// Sign-in cookies, named as the old app names them: both apps talk to the same backend.
export const ACCESS_COOKIE = 'access_token';
export const REFRESH_COOKIE = 'refresh_token';

/** How long a sign-in lasts on a device without being used (the API's refresh tokens move their
 * expiry out on every use). Without a max age they'd be session cookies, lost when the browser
 * closes. */
export const SESSION_MAX_AGE = 30 * 24 * 60 * 60;

/** Refresh this long before the access token expires, so no page load starts on one about to lapse. */
const REFRESH_MARGIN_MS = 60_000;

const base = { path: '/', sameSite: 'lax' as const, maxAge: SESSION_MAX_AGE };

/** When an access token (a JWT) expires, in ms, or null when it can't be read. Only the expiry is
 * read; the token is never trusted for anything else here (the API checks it). */
export function tokenExpiry(token: string): number | null {
	const payload = token.split('.')[1];
	if (!payload) return null;
	try {
		const { exp } = JSON.parse(Buffer.from(payload, 'base64url').toString('utf8')) as { exp?: unknown };
		return typeof exp === 'number' ? exp * 1000 : null;
	} catch {
		return null;
	}
}

/** Whether to get a new access token before using this one. */
export function needsRefresh(token: string | undefined, now: number = Date.now()): boolean {
	if (!token) return true;
	const expiry = tokenExpiry(token);
	return expiry !== null && expiry - now < REFRESH_MARGIN_MS;
}

/** Keeps the person signed in on this device. */
export function setSession(cookies: Cookies, session: { access_token: string; refresh_token?: string }): void {
	cookies.set(ACCESS_COOKIE, session.access_token, { ...base, httpOnly: true });
	if (session.refresh_token) cookies.set(REFRESH_COOKIE, session.refresh_token, { ...base, httpOnly: true });
}

export function clearSession(cookies: Cookies): void {
	cookies.delete(ACCESS_COOKIE, { path: '/' });
	cookies.delete(REFRESH_COOKIE, { path: '/' });
}
