import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

/** Google sends people back here after signing in (or adding Google to their account). */
export const GET: RequestHandler = async ({ url, cookies, fetch }) => {
	// Already signed in means this was "Link Google" from Account settings.
	const errorPage = cookies.get('access_token') ? '/account' : '/login';
	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');
	if (url.searchParams.get('error') || !code || !state) redirect(303, `${errorPage}?google_error=cancelled`);

	const response = await fetch(apiUrl('/api/auth/google/callback'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ code, state }),
	});
	if (!response.ok) {
		const message = (await response.text()).slice(0, 160) || 'failed';
		redirect(303, `${errorPage}?google_error=${encodeURIComponent(message)}`);
	}
	const result = (await response.json()) as {
		email: string;
		is_new: boolean;
		linked: boolean;
		access_token: string | null;
		refresh_token: string | null;
	};
	if (result.linked || !result.access_token || !result.refresh_token) redirect(303, '/account?google=linked');

	cookies.set('access_token', result.access_token, { httpOnly: true, path: '/', sameSite: 'lax' });
	cookies.set('refresh_token', result.refresh_token, { httpOnly: true, path: '/', sameSite: 'lax' });
	cookies.set('user_email', result.email, { httpOnly: false, path: '/', sameSite: 'lax' });
	// New here: offer to let Workspace use this same Google account.
	redirect(303, result.is_new ? '/connections?welcome=1' : '/');
};
