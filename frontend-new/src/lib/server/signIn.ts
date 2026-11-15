import type { Cookies } from '@sveltejs/kit';
import type { ApiEvent } from './api';
import { settleLanguageAfterSignIn } from './locale';
import { setSession } from './session';

/** Where a sign-in that's waiting for its emailed code stands (from the API). */
export type Verification = {
	challenge_id: string;
	purpose: 'login' | 'register';
	email: string;
	expires_in: number;
	resend_in: number;
	sends_left: number;
};

export type TokenPair = { access_token: string; refresh_token: string };

/** What the code buys: the tokens, and whether the code just made the account. */
export type Verified = TokenPair & { is_new?: boolean };

/**
 * The sign-in waiting on the code page: kept server-side only, for 15 minutes. `from` is the form
 * it started on, for "change" to go back to (a new account made on the register form otherwise).
 */
export type PendingSignIn = { challengeId: string; email: string; isNew: boolean; redirectTo: string; from?: 'sign-in' | 'register' };

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
export async function completeSignIn(event: ApiEvent, tokens: TokenPair, isNew: boolean): Promise<void> {
	setSession(event.cookies, tokens);
	event.locals.accessToken = tokens.access_token;
	await settleLanguageAfterSignIn(event, isNew);
}
