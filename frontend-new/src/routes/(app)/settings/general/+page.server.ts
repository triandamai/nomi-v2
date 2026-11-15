import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { DeviceItem } from '$lib/adapters/api';
import { loadGeneral, saveReminders, signOutDevices, startGoogleLink, unlinkGoogle } from '$lib/server/general';
import { parseQuiet } from '$lib/settings/general';
import { savePreferences as saveEmails } from '$lib/server/inbox';
import { displayEmail } from '$lib/auth/format';
import { emailProblem } from '$lib/server/authFlow';
import { apiFetch } from '$lib/server/api';
import { clearSession } from '$lib/server/session';
import type { Verification } from '$lib/server/signIn';
import { readApiError } from '$lib/server/apiError';
import type { Actions, PageServerLoad } from './$types';

/**
 * Settings › General (App / Settings / General m7lvY1): how the person signs in, their language,
 * zone and theme, which emails they get, and their devices. Every change saves as it's made
 * (`autosave` puts that in the top bar); the theme, language and zone through /preferences.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:general');
	const data = await loadGeneral(event);
	if (!data) error(503, { message: m.general_load_failed() });
	return { title: m.settings_general(), autosave: true, ...data };
};

/** A code sent for the account (a new address, closing it): what the dialog says about it. */
const sentCode = (verification: Verification) => ({
	code: { challengeId: verification.challenge_id, email: displayEmail(verification.email), minutes: Math.max(1, Math.ceil(verification.expires_in / 60)) },
});

/** The backend's answer to a wrong, used-up or locked code, on the code's field. */
async function codeFailure(response: Response) {
	const error = await readApiError(response);
	if (error.code === 'wrong_code') {
		const left = error.attemptsLeft ?? 0;
		return fail(400, { field: 'code', message: left === 1 ? m.verify_wrong_last() : m.verify_wrong({ left }) });
	}
	if (error.code === 'too_many_attempts') return fail(410, { field: 'code', message: m.verify_locked(), ended: true });
	if (error.code === 'expired' || response.status === 410 || response.status === 404) return fail(410, { field: 'code', message: m.verify_expired(), ended: true });
	return null;
}

const sixDigits = (form: FormData) => String(form.get('code') ?? '').replace(/\D/g, '');

/** Calls the backend; one that can't be reached answers 503. */
const call = (event: Parameters<typeof apiFetch>[0], path: string, init: RequestInit) => apiFetch(event, path, init).catch(() => null);

export const actions: Actions = {
	/** Changing the email: a code to the new address (`POST /api/account/email`). */
	emailStart: async (event) => {
		const email = String((await event.request.formData()).get('email') ?? '').trim();
		const problem = emailProblem(email);
		if (problem) return fail(400, { field: 'email', message: problem === 'email_required' ? m.err_email_required() : m.err_email_invalid() });
		const response = await call(event, '/api/account/email', { method: 'POST', body: JSON.stringify({ email }) });
		if (!response) return fail(503, { message: m.action_failed() });
		if (response.ok) return sentCode((await response.json()) as Verification);
		const { code } = await readApiError(response);
		if (code === 'email_taken') return fail(409, { field: 'email', message: m.register_email_taken() });
		if (code === 'same_email') return fail(400, { field: 'email', message: m.general_email_same() });
		if (code === 'invalid_email') return fail(400, { field: 'email', message: m.err_email_invalid() });
		if (code === 'no_email_account') return fail(404, { message: m.general_email_no_account() });
		if (response.status === 502) return fail(502, { message: m.verify_email_failed() });
		return fail(response.status, { message: m.action_failed() });
	},
	/** The code from the new address moves the account to it (`POST /api/account/email/verify`). */
	emailVerify: async (event) => {
		const form = await event.request.formData();
		const code = sixDigits(form);
		if (code.length !== 6) return fail(400, { field: 'code', message: m.verify_incomplete() });
		const response = await call(event, '/api/account/email/verify', { method: 'POST', body: JSON.stringify({ challenge_id: String(form.get('challenge_id') ?? ''), code }) });
		if (!response) return fail(503, { message: m.action_failed() });
		if (response.ok) return { email: ((await response.json()) as { email: string }).email };
		const failed = await codeFailure(response.clone());
		if (failed) return failed;
		if ((await readApiError(response)).code === 'email_taken') return fail(409, { field: 'code', message: m.register_email_taken(), ended: true });
		return fail(response.status, { message: m.action_failed() });
	},
	/** Closing an account without its password: a code to its address. */
	deleteCode: async (event) => {
		const response = await call(event, '/api/account/delete/code', { method: 'POST' });
		if (!response) return fail(503, { message: m.action_failed() });
		if (response.ok) return sentCode((await response.json()) as Verification);
		if (response.status === 502) return fail(502, { message: m.verify_email_failed() });
		return fail(response.status, { message: (await readApiError(response)).code === 'no_email_account' ? m.general_email_no_account() : m.action_failed() });
	},
	/**
	 * Closes the account (`DELETE /api/account`) with the password or the emailed code; this device
	 * is signed out and lands on sign in, which says it's closed. A workspace only they own, with
	 * other people in it, has to be handed over first: those come back as `soleOwner`.
	 */
	deleteAccount: async (event) => {
		const form = await event.request.formData();
		const password = String(form.get('password') ?? '');
		const challengeId = String(form.get('challenge_id') ?? '');
		const code = sixDigits(form);
		if (!password && !(challengeId && code.length === 6))
			return challengeId ? fail(400, { field: 'code', message: m.verify_incomplete() }) : fail(400, { field: 'password', message: m.err_password_required() });
		const body = password ? { password } : { challenge_id: challengeId, code };
		const response = await call(event, '/api/account', { method: 'DELETE', body: JSON.stringify(body) });
		if (!response) return fail(503, { message: m.action_failed() });
		if (response.ok) {
			clearSession(event.cookies);
			redirect(303, '/sign-in?closed=1');
		}
		if (response.status === 409) {
			const said = (await response.clone().json().catch(() => ({}))) as { error?: string; orgs?: { id: string; name: string }[] };
			if (said.error === 'sole_owner') return fail(409, { soleOwner: said.orgs ?? [] });
		}
		if (response.status === 403) return fail(403, { field: 'password', message: m.general_password_wrong() });
		const failed = await codeFailure(response);
		return failed ?? fail(response.status, { message: m.action_failed() });
	},

	/** Off to Google; it comes back to this page through /auth/google/callback. */
	linkGoogle: async (event) => {
		const started = await startGoogleLink(event, event.url.origin);
		if (!started.ok) return fail(started.status, { message: started.message });
		redirect(303, started.data.url);
	},
	unlinkGoogle: async (event) => {
		const done = await unlinkGoogle(event);
		return done.ok ? { unlinked: true } : fail(done.status, { message: done.message });
	},
	/** Every device but this one (the backend marks it `current` from this device's id). */
	signOutOthers: async (event) => {
		const response = await apiFetch(event, '/api/devices');
		if (!response.ok) return fail(response.status, { message: m.action_failed() });
		const others = ((await response.json()) as DeviceItem[]).filter((device) => !device.current).map((device) => device.id);
		const done = await signOutDevices(event, others);
		return done.ok ? { signedOut: others.length } : fail(done.status, { message: done.message });
	},
	/**
	 * Sets or changes the password while signed in (`PUT /api/auth/password`): the current one
	 * when there is one. Other devices stay signed in. Each refusal names the field it's about.
	 */
	password: async (event) => {
		const form = await event.request.formData();
		const current = String(form.get('current') ?? '');
		const next = String(form.get('next') ?? '');
		if (next.length < 8) return fail(400, { field: 'next', message: m.err_password_short() });
		if (String(form.get('confirm') ?? '') !== next) return fail(400, { field: 'confirm', message: m.err_password_mismatch() });
		const response = await apiFetch(event, '/api/auth/password', {
			method: 'PUT',
			body: JSON.stringify({ new_password: next, ...(current && { current_password: current }) }),
		}).catch(() => null);
		if (!response) return fail(503, { message: m.action_failed() });
		if (response.ok) return { password: current ? 'changed' : 'set' };
		const { code } = await readApiError(response);
		if (code === 'wrong_password') return fail(403, { field: 'current', message: m.general_password_wrong() });
		if (code === 'too_short') return fail(400, { field: 'next', message: m.err_password_short() });
		return fail(response.status, { message: m.action_failed() });
	},
	/** Reminder pushes (`push_reminders`), or quiet hours (`quiet`: "22:00-06:00", or "off"). */
	reminders: async (event) => {
		const form = await event.request.formData();
		const push = form.get('push_reminders');
		const quiet = form.get('quiet');
		const hours = typeof quiet === 'string' && quiet !== 'off' ? parseQuiet(quiet) : null;
		if (quiet !== null && quiet !== 'off' && !hours) return fail(400, { message: m.action_failed() });
		const change = { ...(push !== null && { push_reminders: push === 'true' }), ...(quiet !== null && { quiet_hours: hours }) };
		if (!Object.keys(change).length) return fail(400, { message: m.action_failed() });
		const done = await saveReminders(event, change);
		return done.ok ? { reminders: done.data } : fail(done.status, { message: done.message });
	},
	emails: async (event) => {
		const form = await event.request.formData();
		const done = await saveEmails(event, { email_account: form.get('email_account') === 'true', email_promos: form.get('email_promos') === 'true' });
		return done.ok ? { emails: done.data } : fail(done.status, { message: done.message });
	},
};
