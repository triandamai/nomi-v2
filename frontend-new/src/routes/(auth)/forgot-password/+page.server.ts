import { fail } from '@sveltejs/kit';
import { mailApp } from '$lib/auth/format';
import type { Alert, Fields } from '$lib/auth/problems';
import { getLocale } from '$lib/paraglide/runtime';
import { callAuth, emailProblem } from '$lib/server/authFlow';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = ({ url }) => ({ email: url.searchParams.get('email') ?? '' });

const failure = (status: number, email: string, fields: Fields, alert: Alert | null = null) => fail(status, { email, fields, alert });

export const actions: Actions = {
	default: async ({ request, fetch }) => {
		const email = String((await request.formData()).get('email') ?? '').trim();
		const problem = emailProblem(email);
		if (problem) return failure(400, email, { email: problem });

		const response = await callAuth(fetch, '/api/auth/forgot', { email, language: getLocale() });
		if (!response) return failure(503, email, {}, { kind: 'unreachable' });
		if (!response.ok) return failure(response.status, email, {}, { kind: 'forgot_failed', status: response.status });
		// The API answers the same whether or not the address has an account.
		return { sent: email, mailApp: mailApp(email) };
	},
};
