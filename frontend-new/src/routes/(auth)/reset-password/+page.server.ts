import { fail, redirect } from '@sveltejs/kit';
import type { Alert, Fields } from '$lib/auth/problems';
import { callAuth, newPasswordProblem } from '$lib/server/authFlow';
import { clearSession } from '$lib/server/session';
import type { Actions, PageServerLoad } from './$types';

// The emailed reset link (`/reset-password?token=`): a new password, which signs the account out
// everywhere. A link the backend says is spent shows as expired; a hiccup lets them try anyway.

export const load: PageServerLoad = async ({ url, fetch }) => {
	const token = url.searchParams.get('token') ?? '';
	if (!token) return { token, expired: true };
	const response = await callAuth(fetch, `/api/auth/reset/${encodeURIComponent(token)}`);
	return { token, expired: response?.status === 410 };
};

const failure = (status: number, fields: Fields, alert: Alert | null = null, expired = false) => fail(status, { fields, alert, expired });

export const actions: Actions = {
	default: async ({ request, fetch, cookies }) => {
		const data = await request.formData();
		const token = String(data.get('token') ?? '');
		const password = String(data.get('password') ?? '');
		const confirm = String(data.get('confirm') ?? '');
		if (!token) return failure(410, {}, null, true);

		const tooWeak = newPasswordProblem(password);
		if (tooWeak) return failure(400, { password: tooWeak });
		if (password !== confirm) return failure(400, { confirm: 'password_mismatch' });

		const response = await callAuth(fetch, '/api/auth/reset', { token, password });
		if (!response) return failure(503, {}, { kind: 'unreachable' });
		if (response.status === 410) return failure(410, {}, null, true);
		if (response.status === 400) return failure(400, { password: 'password_short' });
		if (!response.ok) return failure(response.status, {}, { kind: 'reset_failed', status: response.status });

		// Every device was signed out, this one too: sign in again with the new password.
		clearSession(cookies);
		redirect(303, '/sign-in?reset=1');
	},
};
