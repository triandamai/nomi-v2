import { fail, redirect } from '@sveltejs/kit';
import { displayEmail, mailApp } from '$lib/auth/format';
import type { CodeProblem } from '$lib/auth/problems';
import { withRedirect } from '$lib/auth/redirects';
import { readApiError } from '$lib/server/apiError';
import { callAuth, rememberMethod } from '$lib/server/authFlow';
import { clearPendingSignIn, completeSignIn, getPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';

// Enter code: the emailed code finishes a sign-in or a new account. The sign-in waiting for it is
// kept server-side (the pending cookie), so a reload finds it again through the challenge.

export const load: PageServerLoad = async ({ cookies, fetch, locals }) => {
	const pending = getPendingSignIn(cookies);
	if (!pending) redirect(303, locals.accessToken ? '/' : '/sign-in');
	const response = await callAuth(fetch, `/api/auth/challenge/${encodeURIComponent(pending.challengeId)}`);
	const base = { isNew: pending.isNew };
	if (response?.ok) {
		const challenge = (await response.json()) as Verification;
		const email = displayEmail(challenge.email);
		return {
			...base,
			expired: false,
			email,
			minutes: Math.max(1, Math.ceil(challenge.expires_in / 60)),
			resendIn: challenge.resend_in,
			sendsLeft: challenge.sends_left,
			mailApp: mailApp(email),
		};
	}
	// Used, expired or locked: start over for a new one. Any other failure lets them try the code.
	const expired = response?.status === 410 || response?.status === 404;
	return { ...base, expired, email: pending.email, minutes: null, resendIn: 0, sendsLeft: 1, mailApp: mailApp(pending.email) };
};

type CodeFailure = { problem: CodeProblem; attemptsLeft?: number; seconds?: number; sendsLeft?: number };
const failure = (status: number, data: CodeFailure) => fail(status, data);

export const actions: Actions = {
	verify: async (event) => {
		const pending = getPendingSignIn(event.cookies);
		// Already finished (the code went in twice): carry on.
		if (!pending) redirect(303, event.locals.accessToken ? '/' : '/sign-in');
		const code = String((await event.request.formData()).get('code') ?? '').replace(/\D/g, '');
		if (code.length !== 6) return failure(400, { problem: 'incomplete' });

		const response = await callAuth(event.fetch, '/api/auth/verify', { challenge_id: pending.challengeId, code });
		if (!response) return failure(503, { problem: 'unreachable' });
		if (response.ok) {
			clearPendingSignIn(event.cookies);
			await completeSignIn(event, (await response.json()) as TokenPair, pending.isNew);
			rememberMethod(event.cookies, 'email');
			// A new account names itself first (First run), then goes where it was going.
			redirect(303, pending.isNew ? withRedirect('/first-run', pending.redirectTo) : pending.redirectTo || '/');
		}
		const error = await readApiError(response);
		if (error.code === 'wrong_code') return failure(400, { problem: 'wrong_code', attemptsLeft: error.attemptsLeft ?? 0 });
		if (error.code === 'too_many_attempts') return failure(410, { problem: 'locked' });
		if (error.code === 'expired' || response.status === 410) return failure(410, { problem: 'expired' });
		return failure(response.status, { problem: 'failed' });
	},

	resend: async (event) => {
		const pending = getPendingSignIn(event.cookies);
		if (!pending) redirect(303, '/sign-in');
		const response = await callAuth(event.fetch, '/api/auth/resend', { challenge_id: pending.challengeId });
		if (!response) return failure(503, { problem: 'unreachable' });
		if (response.ok) {
			const challenge = (await response.json()) as Verification;
			return { resent: true, resendIn: challenge.resend_in, sendsLeft: challenge.sends_left };
		}
		const error = await readApiError(response);
		if (error.code === 'wait') return failure(429, { problem: 'wait', seconds: error.seconds ?? 60 });
		if (error.code === 'too_many_codes') return failure(429, { problem: 'too_many_codes', sendsLeft: 0 });
		if (error.code === 'expired' || response.status === 410 || response.status === 404) return failure(410, { problem: 'expired' });
		if (response.status === 502) return failure(502, { problem: 'email_failed' });
		return failure(response.status, { problem: 'failed' });
	},

	/** "change": back to the form, to start again with another address. */
	cancel: async ({ cookies }) => {
		const pending = getPendingSignIn(cookies);
		clearPendingSignIn(cookies);
		redirect(303, withRedirect(pending?.isNew ? '/register' : '/sign-in', pending?.redirectTo));
	},
};
