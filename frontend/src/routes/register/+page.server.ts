import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { Actions } from './$types';

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
			return fail(400, { error: 'Email, password, and organization name are required.' });
		}

		const response = await fetch(apiUrl('/api/auth/register'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password, org: { mode: 'create', name: orgName } }),
		});

		if (response.status === 409) {
			return fail(409, { error: 'That email is already registered.' });
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Registration failed. Please try again.' });
		}

		const { access_token, refresh_token } = (await response.json()) as {
			access_token: string;
			refresh_token: string;
		};

		cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('refresh_token', refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('user_email', email, { httpOnly: false, path: '/', sameSite: 'lax' });

		throw redirect(303, '/');
	},
};
