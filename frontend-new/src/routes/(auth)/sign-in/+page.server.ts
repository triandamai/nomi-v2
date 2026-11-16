import { fail, redirect, type RequestEvent } from '@sveltejs/kit';
import { displayEmail } from '$lib/auth/format';
import { googleErrorFrom, type Alert, type Fields } from '$lib/auth/problems';
import { inviteCodeFrom } from '$lib/auth/redirects';
import { getLocale } from '$lib/paraglide/runtime';
import { readApiError } from '$lib/server/apiError';
import { alertFor, callAuth, emailProblem, googleAvailable, lastMethod, rememberMethod } from '$lib/server/authFlow';
import { completeSignIn, safeRedirect, setPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';

// Auth / Sign in or create (vDaAQ): one step for both. The email alone gets a code
// (`POST /api/auth/start`): an address with an account signs in with it, a new one is set up by
// it. "Use a password instead" (`?with=password`) is the email and password form, which goes on to
// the same code where the server asks for one.

type Method = 'code' | 'password';

/** Where to go after signing in, when it's somewhere other than home. */
function goingTo(url: URL): string | null {
	const target = safeRedirect(url.searchParams.get('redirect_to'));
	return target === '/' ? null : target;
}

export const load: PageServerLoad = async ({ url, cookies, locals, fetch }) => {
	if (locals.accessToken) redirect(303, safeRedirect(url.searchParams.get('redirect_to')));
	// Back from choosing a new password: every device was signed out.
	const passwordChanged = url.searchParams.get('reset') === '1';
	return {
		method: (url.searchParams.get('with') === 'password' || passwordChanged ? 'password' : 'code') as Method,
		google: await googleAvailable(fetch),
		googleError: googleErrorFrom(url.searchParams.get('google_error')),
		lastUsed: lastMethod(cookies),
		passwordChanged,
		// Back from closing the account (Settings › General).
		accountClosed: url.searchParams.get('closed') === '1',
		email: url.searchParams.get('email') ?? '',
		redirectTo: goingTo(url),
	};
};

type Failure = { email: string; method: Method; fields: Fields; alert: Alert | null };
const failure = (status: number, email: string, method: Method, fields: Fields, alert: Alert | null = null) =>
	fail(status, { email, method, fields, alert } satisfies Failure);

/** Off to the code page with the sign-in that waits for it. */
function toCode(event: RequestEvent, verification: Verification): never {
	setPendingSignIn(event.cookies, {
		challengeId: verification.challenge_id,
		email: displayEmail(verification.email),
		isNew: verification.purpose === 'register',
		redirectTo: safeRedirect(event.url.searchParams.get('redirect_to')),
		from: 'sign-in',
	});
	redirect(303, '/verify');
}

async function withCode(event: RequestEvent, email: string) {
	const emailWrong = emailProblem(email);
	if (emailWrong) return failure(400, email, 'code', { email: emailWrong });

	// A new address that came from an invite joins that workspace as its account is made.
	const invite = inviteCodeFrom(safeRedirect(event.url.searchParams.get('redirect_to')));
	const ask = (invite_code?: string) => callAuth(event.fetch, '/api/auth/start', { email, language: getLocale(), ...(invite_code && { invite_code }) });
	let response = await ask(invite ?? undefined);
	if (!response) return failure(503, email, 'code', {}, { kind: 'unreachable' });
	if (response.status === 400) {
		const error = await readApiError(response.clone());
		if (error.code === 'invalid_email') return failure(400, email, 'code', { email: 'email_invalid' });
		// An invite that ended doesn't stop the account: the invite's page says it ended.
		if (error.code === 'invalid_invite' && invite) {
			response = await ask();
			if (!response) return failure(503, email, 'code', {}, { kind: 'unreachable' });
		}
	}
	if (!response.ok) {
		const alert = await alertFor(response, 'sign_in_failed');
		return failure(response.status, email, 'code', {}, alert.kind === 'email_failed' ? { ...alert, kind: 'code_email_failed' } : alert);
	}
	toCode(event, ((await response.json()) as { verification: Verification }).verification);
}

async function withPassword(event: RequestEvent, email: string, password: string) {
	const fields: Fields = {};
	const emailWrong = emailProblem(email);
	if (emailWrong) fields.email = emailWrong;
	if (!password) fields.password = 'password_required';
	if (Object.keys(fields).length) return failure(400, email, 'password', fields);

	const response = await callAuth(event.fetch, '/api/auth/login', { email, password });
	if (!response) return failure(503, email, 'password', {}, { kind: 'unreachable' });
	if (response.status === 401) return failure(401, email, 'password', { password: 'wrong_credentials' });
	if (!response.ok) return failure(response.status, email, 'password', {}, await alertFor(response, 'sign_in_failed'));

	const body = (await response.json()) as TokenPair | { verification: Verification };
	// The right password emails a code; signing in finishes on /verify.
	if ('verification' in body) toCode(event, body.verification);
	await completeSignIn(event, body, false);
	rememberMethod(event.cookies, 'email');
	redirect(303, safeRedirect(event.url.searchParams.get('redirect_to')));
}

export const actions: Actions = {
	default: async (event) => {
		const data = await event.request.formData();
		const email = String(data.get('email') ?? '').trim();
		return data.get('method') === 'password' ? withPassword(event, email, String(data.get('password') ?? '')) : withCode(event, email);
	},
};
