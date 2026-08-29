import { fail, redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { Actions } from './$types';

export const actions: Actions = {
	default: async ({ request, cookies, fetch, url }) => {
		const data = await request.formData();
		const email = data.get('email');
		const password = data.get('password');

		if (typeof email !== 'string' || typeof password !== 'string' || !email || !password) {
			return fail(400, { error: 'Email and password are required.' });
		}

		const response = await fetch(apiUrl('/api/auth/login'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ email, password }),
		});

		if (response.status === 401) {
			return fail(401, { error: 'Invalid email or password.' });
		}
		if (!response.ok) {
			return fail(response.status, { error: 'Login failed. Please try again.' });
		}

		const { access_token, refresh_token } = (await response.json()) as {
			access_token: string;
			refresh_token: string;
		};

		cookies.set('access_token', access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('refresh_token', refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
		cookies.set('user_email', email, { httpOnly: false, path: '/', sameSite: 'lax' });

		const redirectTo = url.searchParams.get('redirect_to');
		throw redirect(303, redirectTo && redirectTo.startsWith('/') ? redirectTo : '/');
	},
};
