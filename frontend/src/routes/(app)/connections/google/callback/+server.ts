import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

/** Google sends the user back here after they allow (or decline) access. */
export const GET: RequestHandler = async ({ url, cookies, fetch }) => {
	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');
	if (url.searchParams.get('error') || !code || !state) {
		redirect(303, '/connections?error=declined');
	}
	const response = await apiFetch(fetch, cookies, '/api/connections/google/callback', {
		method: 'POST',
		body: JSON.stringify({ code, state }),
	});
	if (!response.ok) {
		redirect(303, `/connections?error=${encodeURIComponent((await response.text()).slice(0, 160) || 'failed')}`);
	}
	const { resume_session_id } = (await response.json()) as { resume_session_id: string | null };
	redirect(303, resume_session_id ? `/chat/${resume_session_id}` : '/connections?connected=1');
};
