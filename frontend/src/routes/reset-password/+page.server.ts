import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

/** The shortest password the API takes for a reset (it checks too). */
const MIN_PASSWORD_LENGTH = 8;

export const load: PageServerLoad = async ({ fetch, url }) => {
	const token = url.searchParams.get('token') ?? '';
	if (!token) return { token, expired: true };
	const response = await fetch(apiUrl(`/api/auth/reset/${encodeURIComponent(token)}`)).catch(() => null);
	// Only a link the API says is spent counts as expired; a hiccup lets them try anyway.
	return { token, expired: response?.status === 410 };
};

export const actions: Actions = {
	default: async ({ request, fetch }) => {
		const data = await request.formData();
		const token = data.get('token');
		const password = data.get('password');
		const confirm = data.get('confirmPassword');

		if (typeof token !== 'string' || !token) {
			return fail(410, { error: m.reset_expired(), expired: true, field: null });
		}
		if (typeof password !== 'string' || password.length < MIN_PASSWORD_LENGTH) {
			return fail(400, { error: m.err_password_short(), field: 'password' });
		}
		if (password !== confirm) {
			return fail(400, { error: m.err_password_mismatch(), field: 'confirm' });
		}

		const response = await fetch(apiUrl('/api/auth/reset'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ token, password }),
		}).catch(() => null);

		if (response?.status === 410) return fail(410, { error: m.reset_expired(), expired: true, field: null });
		if (response?.status === 400) return fail(400, { error: m.err_password_short(), field: 'password' });
		if (!response?.ok) return fail(response?.status ?? 502, { error: m.reset_failed(), field: null });

		// Every device was signed out: log in again with the new password.
		throw redirect(303, '/login?reset=1');
	},
};
