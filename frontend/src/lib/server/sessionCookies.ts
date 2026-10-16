import type { Cookies } from '@sveltejs/kit';

/** How long a sign-in lasts on a device without being used (matches the API's refresh tokens,
 * whose expiry moves out on every use). Without a max age the cookies were session cookies, which
 * browsers drop when they close, so people had to sign in again and again. */
export const SESSION_MAX_AGE = 30 * 24 * 60 * 60;

const base = { path: '/', sameSite: 'lax' as const, maxAge: SESSION_MAX_AGE };

/** Keeps the person signed in on this device. */
export function setSession(cookies: Cookies, session: { access_token: string; refresh_token?: string; email?: string }): void {
	cookies.set('access_token', session.access_token, { ...base, httpOnly: true });
	if (session.refresh_token) cookies.set('refresh_token', session.refresh_token, { ...base, httpOnly: true });
	if (session.email !== undefined) cookies.set('user_email', session.email, { ...base, httpOnly: false });
}

export function clearSession(cookies: Cookies): void {
	cookies.delete('access_token', { path: '/' });
	cookies.delete('refresh_token', { path: '/' });
}
