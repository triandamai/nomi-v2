import type { Cookies } from '@sveltejs/kit';
import { settleLanguageAfterSignIn } from '$lib/server/locale';

/** Where a password sign-in that's waiting for its emailed code stands (from the API). */
export type Verification = {
	challenge_id: string;
	purpose: 'login' | 'register';
	email: string;
	expires_in: number;
	resend_in: number;
	sends_left: number;
};

export type TokenPair = { access_token: string; refresh_token: string };

/** The sign-in waiting on the code page: kept server-side only, for 15 minutes. */
export type PendingSignIn = { challengeId: string; email: string; isNew: boolean; redirectTo: string };

const PENDING_COOKIE = 'pending_sign_in';

export function setPendingSignIn(cookies: Cookies, pending: PendingSignIn): void {
	cookies.set(PENDING_COOKIE, JSON.stringify(pending), { httpOnly: true, path: '/', sameSite: 'lax', maxAge: 15 * 60 });
}

export function getPendingSignIn(cookies: Cookies): PendingSignIn | null {
	const raw = cookies.get(PENDING_COOKIE);
	if (!raw) return null;
	try {
		const value = JSON.parse(raw) as PendingSignIn;
		return typeof value.challengeId === 'string' && typeof value.email === 'string' ? value : null;
	} catch {
		return null;
	}
}

export function clearPendingSignIn(cookies: Cookies): void {
	cookies.delete(PENDING_COOKIE, { path: '/' });
}

/** Only paths on this site: never send someone elsewhere after signing in. */
export function safeRedirect(target: string | null | undefined): string {
	return target && target.startsWith('/') && !target.startsWith('//') ? target : '/';
}

/** Signs the person in on this device with the tokens the API issued. */
export async function completeSignIn(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	tokens: TokenPair,
	email: string,
	isNew: boolean,
): Promise<void> {
	cookies.set('access_token', tokens.access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
	cookies.set('refresh_token', tokens.refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
	cookies.set('user_email', email, { httpOnly: false, path: '/', sameSite: 'lax' });
	await settleLanguageAfterSignIn(fetch, cookies, isNew);
}
