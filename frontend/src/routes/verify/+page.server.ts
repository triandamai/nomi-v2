import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import { clearPendingSignIn, completeSignIn, getPendingSignIn, type TokenPair, type Verification } from '$lib/server/signIn';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

type ApiError = { error?: string; attempts_left?: number; seconds?: number };

async function post(fetch: typeof globalThis.fetch, path: string, body: unknown) {
	const response = await fetch(apiUrl(path), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(body),
	});
	const json = (await response.json().catch(() => ({}))) as unknown;
	return { status: response.status, ok: response.ok, json };
}

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const pending = getPendingSignIn(cookies);
	if (!pending) throw redirect(303, cookies.get('access_token') ? '/' : '/login');
	const response = await fetch(apiUrl(`/api/auth/challenge/${encodeURIComponent(pending.challengeId)}`)).catch(() => null);
	if (!response?.ok) {
		return { expired: true as const, isNew: pending.isNew, verification: null };
	}
	return { expired: false as const, isNew: pending.isNew, verification: (await response.json()) as Verification };
};

export const actions: Actions = {
	verify: async ({ request, cookies, fetch }) => {
		const pending = getPendingSignIn(cookies);
		// Already finished (e.g. the code went in twice): carry on into the app.
		if (!pending) throw redirect(303, cookies.get('access_token') ? '/' : '/login');
		const code = String((await request.formData()).get('code') ?? '').replace(/\D/g, '');
		if (code.length !== 6) return fail(400, { error: m.verify_six_digits() });

		const result = await post(fetch, '/api/auth/verify', { challenge_id: pending.challengeId, code });
		if (result.ok) {
			clearPendingSignIn(cookies);
			await completeSignIn(fetch, cookies, result.json as TokenPair, pending.email, pending.isNew);
			throw redirect(303, pending.redirectTo || '/');
		}
		const error = result.json as ApiError;
		if (error.error === 'wrong_code') {
			return fail(400, { error: m.verify_wrong({ left: String(error.attempts_left ?? 0) }) });
		}
		if (error.error === 'too_many_attempts') return fail(410, { error: m.verify_locked(), expired: true });
		if (result.status === 410) return fail(410, { error: m.verify_expired(), expired: true });
		return fail(result.status, { error: m.verify_failed() });
	},

	resend: async ({ cookies, fetch }) => {
		const pending = getPendingSignIn(cookies);
		if (!pending) throw redirect(303, '/login');
		const result = await post(fetch, '/api/auth/resend', { challenge_id: pending.challengeId });
		if (result.ok) {
			const verification = result.json as Verification;
			return { resent: true, resendIn: verification.resend_in, sendsLeft: verification.sends_left };
		}
		const error = result.json as ApiError;
		if (error.error === 'wait') return fail(429, { error: m.verify_wait({ seconds: String(error.seconds ?? 60) }), resendIn: error.seconds });
		if (error.error === 'too_many_codes') return fail(429, { error: m.verify_too_many_codes(), sendsLeft: 0 });
		if (result.status === 410) return fail(410, { error: m.verify_expired(), expired: true });
		if (result.status === 502) return fail(502, { error: m.verify_email_failed() });
		return fail(result.status, { error: m.verify_failed() });
	},

	cancel: async ({ cookies }) => {
		const pending = getPendingSignIn(cookies);
		clearPendingSignIn(cookies);
		throw redirect(303, pending?.isNew ? '/register' : '/login');
	},
};
