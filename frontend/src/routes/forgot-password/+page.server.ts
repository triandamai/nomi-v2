import { fail } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import { getLocale } from '$lib/paraglide/runtime';
import type { Actions } from './$types';
import { m } from '$lib/paraglide/messages';

export const actions: Actions = {
	default: async ({ request, fetch }) => {
		const data = await request.formData();
		const email = data.get('email');
		if (typeof email !== 'string' || !email.trim()) {
			return fail(400, { error: m.forgot_email_required(), field: 'email' });
		}

		const response = await fetch(apiUrl('/api/auth/forgot'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email: email.trim(), language: getLocale() }),
		}).catch(() => null);
		if (!response?.ok) {
			return fail(response?.status ?? 502, { error: m.forgot_failed(), field: null });
		}
		// The API answers the same whether or not the address has an account.
		return { sent: email.trim() };
	},
};
