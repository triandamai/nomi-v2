import { fail, redirect } from '@sveltejs/kit';
import { FORMAT_REGIONS } from '$lib/accountSettings';
import { isLanguage } from '$lib/i18n';
import { m } from '$lib/paraglide/messages';
import { apiFetch } from '$lib/server/api';
import { setLanguageCookie } from '$lib/server/locale';
import { clearSession, SESSION_MAX_AGE } from '$lib/server/sessionCookies';
import type { Profile } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export interface SignInMethods {
	configured: boolean;
	password: boolean;
	google_email: string | null;
}

/** One of the person's devices (backend: routes/devices.rs `DeviceItem`). */
export interface Device {
	id: string;
	platform: 'android' | 'ios' | 'macos' | 'windows' | 'linux' | 'web';
	name: string | null;
	brand: string | null;
	model: string | null;
	os_name: string | null;
	os_version: string | null;
	app_version: string;
	presence: 'online' | 'away' | 'offline';
	last_seen_at: string;
	created_at: string;
	/** The browser this page is open in. */
	current: boolean;
}

/** One of the person's live share links (backend: routes/shares.rs `ShareListItem`). */
export interface ShareLink {
	id: string;
	session_id: string;
	title: string | null;
	scope: 'reply' | 'chat';
	message_id: string | null;
	mask: boolean;
	anyone: boolean;
	expires_at: string | null;
	created_at: string;
}

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const [profileResponse, methodsResponse, devicesResponse, sharesResponse] = await Promise.all([
		apiFetch(fetch, cookies, '/api/profile'),
		apiFetch(fetch, cookies, '/api/auth/google'),
		apiFetch(fetch, cookies, '/api/devices'),
		apiFetch(fetch, cookies, '/api/shares'),
	]);
	const profile: Profile | null = profileResponse.ok ? ((await profileResponse.json()) as Profile) : null;
	const methods: SignInMethods | null = methodsResponse.ok ? ((await methodsResponse.json()) as SignInMethods) : null;
	const devices: Device[] | null = devicesResponse.ok ? ((await devicesResponse.json()) as Device[]) : null;
	const shares: ShareLink[] | null = sharesResponse.ok ? ((await sharesResponse.json()) as { shares: ShareLink[] }).shares : null;
	return { profile, methods, devices, shares };
};

export const actions: Actions = {
	/** Adds Google sign-in to this account: off to Google. */
	link: async ({ cookies, fetch, url }) => {
		const response = await apiFetch(fetch, cookies, '/api/auth/google/link', { method: 'POST', body: JSON.stringify({ origin: url.origin }) });
		if (!response.ok) return fail(response.status, { error: m.account_link_failed() });
		const { url: googleUrl } = (await response.json()) as { url: string };
		redirect(303, googleUrl);
	},
	unlink: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/auth/google', { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { error: (await response.text()) || m.account_unlink_failed() });
		return { unlinked: true };
	},
	/** Sets a password (an account made with Google or an emailed code has none), or changes it. */
	password: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const current = String(form.get('current_password') ?? '');
		const next = String(form.get('new_password') ?? '');
		if (!next) return fail(400, { passwordError: m.account_password_too_short() });
		const response = await apiFetch(fetch, cookies, '/api/auth/password', {
			method: 'PUT',
			body: JSON.stringify({ current_password: current || null, new_password: next }),
		});
		if (response.ok) return { passwordSaved: true };
		const { error } = (await response.json().catch(() => ({}))) as { error?: string };
		if (error === 'too_short') return fail(400, { passwordError: m.account_password_too_short() });
		if (error === 'wrong_password') return fail(403, { passwordError: m.account_password_wrong() });
		return fail(response.status, { passwordError: m.account_password_failed() });
	},
	/** Signs one of the person's other devices out: its sign-ins end and it leaves. */
	signOutDevice: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const id = form.get('id');
		const name = String(form.get('name') ?? '');
		if (typeof id !== 'string' || !id) return fail(400, { deviceError: m.account_device_sign_out_failed() });
		const response = await apiFetch(fetch, cookies, `/api/devices/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { deviceError: m.account_device_sign_out_failed() });
		return { signedOut: name };
	},
	/** Settings › Data: one of the person's share links stops working. */
	revokeShare: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const id = String(form.get('id') ?? '');
		const sessionId = String(form.get('session_id') ?? '');
		if (!id || !sessionId) return fail(400, { shareError: m.account_share_revoke_failed() });
		const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(sessionId)}/shares/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!response.ok) return fail(response.status, { shareError: m.account_share_revoke_failed() });
		return { shareRevoked: true };
	},
	/** Change email, step 1: a code goes to the new address. */
	emailStart: async ({ request, cookies, fetch }) => {
		const email = String((await request.formData()).get('email') ?? '').trim();
		if (!email) return fail(400, { emailError: m.account_email_invalid() });
		const response = await apiFetch(fetch, cookies, '/api/account/email', { method: 'POST', body: JSON.stringify({ email }) });
		if (response.ok) {
			const sent = (await response.json()) as { challenge_id: string; email: string };
			return { emailChallenge: sent.challenge_id, emailSentTo: sent.email };
		}
		const { error } = (await response.json().catch(() => ({}))) as { error?: string };
		if (error === 'email_taken') return fail(409, { emailError: m.account_email_taken() });
		if (error === 'same_email') return fail(400, { emailError: m.account_email_same() });
		if (error === 'invalid_email') return fail(400, { emailError: m.account_email_invalid() });
		return fail(response.status, { emailError: codeError(error) ?? m.account_email_failed() });
	},
	/** Change email, step 2: the code from the new address moves the account to it. */
	emailVerify: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const challenge = String(form.get('challenge_id') ?? '');
		const code = String(form.get('code') ?? '').trim();
		if (!challenge || !code) return fail(400, { emailChallenge: challenge, emailError: m.account_code_wrong() });
		const response = await apiFetch(fetch, cookies, '/api/account/email/verify', { method: 'POST', body: JSON.stringify({ challenge_id: challenge, code }) });
		if (response.ok) {
			const { email } = (await response.json()) as { email: string };
			cookies.set('user_email', email, { path: '/', sameSite: 'lax', maxAge: SESSION_MAX_AGE, httpOnly: false });
			return { emailChanged: email };
		}
		const { error } = (await response.json().catch(() => ({}))) as { error?: string };
		// A wrong code can be tried again; anything else starts over.
		if (error === 'wrong_code') return fail(400, { emailChallenge: challenge, emailError: m.account_code_wrong() });
		if (error === 'email_taken') return fail(409, { emailError: m.account_email_taken() });
		return fail(response.status, { emailError: codeError(error) ?? m.account_email_failed() });
	},
	/** Settings › General: formats, who new chats start with, Enter sends, tokens under replies. */
	general: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		const formats = String(form.get('formats') ?? '');
		if (!(FORMAT_REGIONS as readonly string[]).includes(formats)) return fail(400, { generalError: m.account_general_failed() });
		const body = {
			formats,
			default_agent: String(form.get('default_agent') ?? ''),
			enter_sends: form.get('enter_sends') === 'true',
			show_reply_tokens: form.get('show_reply_tokens') === 'true',
		};
		const response = await apiFetch(fetch, cookies, '/api/preferences', { method: 'PUT', body: JSON.stringify(body) });
		if (!response.ok) return fail(response.status, { generalError: m.account_general_failed() });
		return { generalSaved: true };
	},
	/** Delete account without a password: a code goes to the account's address. */
	deleteCode: async ({ cookies, fetch }) => {
		const response = await apiFetch(fetch, cookies, '/api/account/delete/code', { method: 'POST' });
		if (!response.ok) {
			const { error } = (await response.json().catch(() => ({}))) as { error?: string };
			return fail(response.status, { deleteError: codeError(error) ?? m.account_delete_failed() });
		}
		const sent = (await response.json()) as { challenge_id: string; email: string };
		return { deleteChallenge: sent.challenge_id, deleteSentTo: sent.email };
	},
	/** Closes the account with the password or the emailed code, then signs this browser out. */
	deleteAccount: async ({ request, cookies, fetch }) => {
		const form = await request.formData();
		if (form.get('confirm') !== 'on') return fail(400, { deleteError: m.account_delete_confirm_needed() });
		const password = String(form.get('password') ?? '');
		const challenge = String(form.get('challenge_id') ?? '');
		const code = String(form.get('code') ?? '').trim();
		const body = password ? { password } : { challenge_id: challenge, code };
		const response = await apiFetch(fetch, cookies, '/api/account', { method: 'DELETE', body: JSON.stringify(body) });
		if (response.ok) {
			clearSession(cookies);
			cookies.delete('user_email', { path: '/' });
			redirect(303, '/login');
		}
		const failure = (await response.json().catch(() => ({}))) as { error?: string; orgs?: { id: string; name: string }[] };
		const keep = challenge ? { deleteChallenge: challenge } : {};
		if (failure.error === 'sole_owner') {
			return fail(409, { deleteError: m.account_delete_sole_owner({ orgs: (failure.orgs ?? []).map((o) => o.name).join(', ') }) });
		}
		if (failure.error === 'wrong_password') return fail(403, { ...keep, deleteError: m.account_password_wrong() });
		if (failure.error === 'wrong_code') return fail(400, { ...keep, deleteError: m.account_code_wrong() });
		return fail(response.status, { deleteError: codeError(failure.error) ?? m.account_delete_failed() });
	},
	/** The app's language and the crew's (Account → Language, and the admin console's sheet). */
	updateLanguage: async ({ request, cookies, fetch }) => {
		const language = (await request.formData()).get('language');
		if (!isLanguage(language)) return fail(400, { error: m.language_invalid() });
		const response = await apiFetch(fetch, cookies, '/api/preferences', { method: 'PUT', body: JSON.stringify({ language }) });
		if (!response.ok) return fail(response.status, { error: m.language_save_failed() });
		setLanguageCookie(cookies, language);
		return { language };
	},
};

/** What an emailed-code error means for the person, when it's one of those. */
function codeError(error: string | undefined): string | null {
	switch (error) {
		case 'expired':
		case 'too_many_attempts':
			return m.account_code_expired();
		case 'wait':
		case 'too_many_codes':
			return m.account_code_wait();
		case 'email_failed':
			return m.account_code_email_failed();
		default:
			return null;
	}
}
