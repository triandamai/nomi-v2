import { fail, redirect } from '@sveltejs/kit';
import { isLanguage } from '$lib/i18n';
import { m } from '$lib/paraglide/messages';
import { apiFetch } from '$lib/server/api';
import { setLanguageCookie } from '$lib/server/locale';
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
