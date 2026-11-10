import { fail, redirect } from '@sveltejs/kit';
import { displayEmail } from '$lib/auth/format';
import { googleErrorFrom, type Alert, type Fields } from '$lib/auth/problems';
import { alertFor, callAuth, emailProblem, googleAvailable, lastMethod, rememberMethod } from '$lib/server/authFlow';
import { completeSignIn, safeRedirect, setPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';

/** Where to go after signing in, when it's somewhere other than home. */
function goingTo(url: URL): string | null {
	const target = safeRedirect(url.searchParams.get('redirect_to'));
	return target === '/' ? null : target;
}

export const load: PageServerLoad = async ({ url, cookies, locals, fetch }) => {
	if (locals.accessToken) redirect(303, safeRedirect(url.searchParams.get('redirect_to')));
	return {
		google: await googleAvailable(fetch),
		googleError: googleErrorFrom(url.searchParams.get('google_error')),
		lastUsed: lastMethod(cookies),
		// Back from choosing a new password: every device was signed out.
		passwordChanged: url.searchParams.get('reset') === '1',
		email: url.searchParams.get('email') ?? '',
		redirectTo: goingTo(url),
	};
};

type Failure = { email: string; fields: Fields; alert: Alert | null };
const failure = (status: number, email: string, fields: Fields, alert: Alert | null = null) => fail(status, { email, fields, alert } satisfies Failure);

export const actions: Actions = {
	default: async (event) => {
		const data = await event.request.formData();
		const email = String(data.get('email') ?? '').trim();
		const password = String(data.get('password') ?? '');

		const fields: Fields = {};
		const emailWrong = emailProblem(email);
		if (emailWrong) fields.email = emailWrong;
		if (!password) fields.password = 'password_required';
		if (Object.keys(fields).length) return failure(400, email, fields);

		const response = await callAuth(event.fetch, '/api/auth/login', { email, password });
		if (!response) return failure(503, email, {}, { kind: 'unreachable' });
		if (response.status === 401) return failure(401, email, { password: 'wrong_credentials' });
		if (!response.ok) return failure(response.status, email, {}, await alertFor(response, 'sign_in_failed'));

		const body = (await response.json()) as TokenPair | { verification: Verification };
		const redirectTo = safeRedirect(event.url.searchParams.get('redirect_to'));
		// The right password emails a code; signing in finishes on /verify.
		if ('verification' in body) {
			setPendingSignIn(event.cookies, { challengeId: body.verification.challenge_id, email: displayEmail(body.verification.email), isNew: false, redirectTo });
			redirect(303, '/verify');
		}
		await completeSignIn(event, body, false);
		rememberMethod(event.cookies, 'email');
		redirect(303, redirectTo);
	},
};
