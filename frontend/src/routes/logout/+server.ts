import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ cookies, fetch, url }) => {
	const refreshToken = cookies.get('refresh_token');
	if (refreshToken) {
		await fetch(apiUrl('/api/auth/logout'), {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ refresh_token: refreshToken }),
		});
	}
	cookies.delete('access_token', { path: '/' });
	cookies.delete('refresh_token', { path: '/' });
	cookies.delete('user_email', { path: '/' });

	const requested = url.searchParams.get('redirect_to');
	const redirectTo =
		requested && requested.startsWith('/') && !requested.startsWith('//') && !requested.includes('\\')
			? requested
			: '/login';
	throw redirect(303, redirectTo);
};
