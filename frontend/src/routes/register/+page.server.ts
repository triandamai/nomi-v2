import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import { getLocale } from '$lib/paraglide/runtime';
import { completeSignIn, setPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
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
	default: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const email = data.get('email');
		const password = data.get('password');
		const orgName = data.get('orgName');

		if (
			typeof email !== 'string' ||
			typeof password !== 'string' ||
			typeof orgName !== 'string' ||
			!email ||
			!password ||
			!orgName
		) {
			return fail(400, { error: m.err_register_required() });
		}

		const response = await fetch(apiUrl('/api/auth/register'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password, org: { mode: 'create', name: orgName }, language: getLocale() }),
		});

		if (response.status === 409) {
			return fail(409, { error: m.err_email_taken() });
		}
		if (response.status === 502) {
			// The account exists; signing in sends a new code.
			return fail(502, { error: m.verify_register_email_failed() });
		}
		if (!response.ok) {
			return fail(response.status, { error: m.err_register_failed() });
		}

		const body = (await response.json()) as TokenPair | { verification: Verification };
		// A new account confirms its email with a code first; sign-up finishes on /verify.
		if ('verification' in body) {
			setPendingSignIn(cookies, { challengeId: body.verification.challenge_id, email, isNew: true, redirectTo: '/' });
			throw redirect(303, '/verify');
		}

		await completeSignIn(fetch, cookies, body, email, true);
		throw redirect(303, '/');
	},
};
