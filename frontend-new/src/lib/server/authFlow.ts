import type { Cookies } from '@sveltejs/kit';
import { isEmail } from '$lib/auth/format';
import { MIN_PASSWORD_LENGTH, type Alert, type AlertKind, type FieldProblem, type GoogleError } from '$lib/auth/problems';
import { apiUrl } from './api';
import { readApiError } from './apiError';

// What the sign-in pages share on the server: the public auth calls, Google's availability and
// errors, the way this browser signed in last, and the checks every form makes first.

/**
 * A public auth call (no sign-in needed): JSON in, the response back, or null when the backend
 * can't be reached. It never asks for problem details: that rewrite drops `attempts_left` and
 * `seconds` from the emailed code's errors.
 */
export async function callAuth(fetchFn: typeof fetch, path: string, body?: unknown): Promise<Response | null> {
	const init: RequestInit | undefined = body === undefined ? undefined : { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) };
	return fetchFn(apiUrl(path), init).catch(() => null);
}

/** Whether the server offers "Continue with Google". */
export async function googleAvailable(fetchFn: typeof fetch): Promise<boolean> {
	const response = await callAuth(fetchFn, '/api/auth/google/available');
	if (!response?.ok) return false;
	const body = (await response.json().catch(() => ({}))) as { available?: unknown };
	return body.available === true;
}

/** Why Google came back without a sign-in, from the callback's answer. */
export async function googleErrorFor(response: Response): Promise<GoogleError> {
	if (response.status === 409) return 'conflict';
	if (response.status === 503) return 'unavailable';
	if (response.status !== 400) return 'failed';
	const said = (await response.text().catch(() => '')).toLowerCase();
	if (said.includes('verif')) return 'unverified';
	if (said.includes('invite')) return 'invite';
	return 'expired';
}

/** A failed sign-in step as the alert above its form: too many codes, the email not sent, or else `fallback`. */
export async function alertFor(response: Response, fallback: AlertKind): Promise<Alert> {
	const error = await readApiError(response);
	const kind: AlertKind =
		error.code === 'too_many_codes' || response.status === 429 ? 'too_many_codes' : error.code === 'email_failed' || response.status === 502 ? 'email_failed' : fallback;
	return { kind, status: response.status, ...(error.code && { code: error.code }) };
}

export type SignInMethod = 'google' | 'email';

/** How this browser signed in last, for the "last used" mark. */
export const LAST_METHOD_COOKIE = 'nm_last_sign_in';

export function rememberMethod(cookies: Cookies, method: SignInMethod): void {
	cookies.set(LAST_METHOD_COOKIE, method, { path: '/', httpOnly: true, sameSite: 'lax', maxAge: 365 * 24 * 60 * 60 });
}

export function lastMethod(cookies: Cookies): SignInMethod | null {
	const method = cookies.get(LAST_METHOD_COOKIE);
	return method === 'google' || method === 'email' ? method : null;
}

/** Where to go after Google, kept while the person is away at Google (10 minutes). */
export const GOOGLE_RETURN_COOKIE = 'google_return_to';

export function emailProblem(email: string): FieldProblem | null {
	if (!email) return 'email_required';
	return isEmail(email) ? null : 'email_invalid';
}

export function newPasswordProblem(password: string): FieldProblem | null {
	if (!password) return 'password_required';
	return password.length < MIN_PASSWORD_LENGTH ? 'password_short' : null;
}
