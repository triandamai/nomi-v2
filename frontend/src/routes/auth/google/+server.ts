import { redirect } from '@sveltejs/kit';
import { apiUrl } from '$lib/server/api';
import type { RequestHandler } from './$types';

/** "Continue with Google" on the sign-in and sign-up pages: off to Google's sign-in. */
export const POST: RequestHandler = async ({ request, fetch }) => {
	const data = await request.formData();
	const inviteCode = String(data.get('invite_code') ?? '').trim() || null;
	const response = await fetch(apiUrl('/api/auth/google/start'), {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify({ invite_code: inviteCode }),
	});
	if (!response.ok) redirect(303, '/login?google_error=unavailable');
	const { url } = (await response.json()) as { url: string };
	redirect(303, url);
};
