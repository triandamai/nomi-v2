import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Whether a username is free, asked while the person types it on their profile (the backend's
// /api/profile/check-username, with their session, so their own name counts as free).
export const GET: RequestHandler = async ({ url, cookies, fetch }) => {
	const username = url.searchParams.get('username') ?? '';
	const response = await apiFetch(fetch, cookies, `/api/profile/check-username?username=${encodeURIComponent(username)}`).catch(() => null);
	if (!response?.ok) return json({ available: null, reason: null }, { status: response?.status ?? 502 });
	return json(await response.json());
};
