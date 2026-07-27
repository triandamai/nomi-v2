import { redirect } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { SessionSummary } from '$lib/types';
import type { LayoutServerLoad } from './$types';

export const load: LayoutServerLoad = async ({ locals, cookies, fetch }) => {
	if (!locals.accessToken) {
		throw redirect(303, '/login');
	}

	const response = await apiFetch(fetch, cookies, '/api/sessions');
	if (!response.ok) {
		throw redirect(303, '/login');
	}

	const { sessions } = (await response.json()) as { sessions: SessionSummary[] };
	const userEmail = cookies.get('user_email') ?? '';

	return { sessions, userEmail };
};
