import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import { completeSignIn, safeRedirect, setPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ fetch, url }) => {
	const response = await fetch(apiUrl('/api/auth/google/available')).catch(() => null);
	const google = response?.ok ? ((await response.json()) as { available: boolean }).available : false;
	const error = url.searchParams.get('google_error');
	const googleError =
		error === 'cancelled' ? m.err_google_cancelled() : error === 'unavailable' ? m.err_google_unavailable() : error;
	return { google, googleError };
};

export const actions: Actions = {
	default: async ({ request, cookies, fetch, url }) => {
		const data = await request.formData();
		const email = data.get('email');
		const password = data.get('password');

		if (typeof email !== 'string' || typeof password !== 'string' || !email || !password) {
			return fail(400, { error: m.err_email_password_required() });
		}

		const response = await fetch(apiUrl('/api/auth/login'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password }),
		});

		if (response.status === 401) {
			return fail(401, { error: m.err_invalid_login() });
		}
		if (response.status === 429) {
			return fail(429, { error: m.verify_too_many_codes() });
		}
		if (response.status === 502) {
			return fail(502, { error: m.verify_email_failed() });
		}
		if (!response.ok) {
			return fail(response.status, { error: m.err_login_failed() });
		}

		const body = (await response.json()) as TokenPair | { verification: Verification };
		const redirectTo = safeRedirect(url.searchParams.get('redirect_to'));

		// The right password sends a code to the account's email; sign-in finishes on /verify.
		if ('verification' in body) {
			setPendingSignIn(cookies, { challengeId: body.verification.challenge_id, email, isNew: false, redirectTo });
			throw redirect(303, '/verify');
		}

		await completeSignIn(fetch, cookies, body, email, false);
		throw redirect(303, redirectTo);
	},
};
