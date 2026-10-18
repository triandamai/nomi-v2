import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { apiFetch } from '$lib/server/api';

/** The person's sessions, newest first, for the drawer's Recents ($lib/drawerRecents). */
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/sessions');
	if (!response.ok) throw error(response.status === 401 ? 401 : 502, 'recent chats unavailable');
	return new Response(await response.text(), { headers: { 'Content-Type': 'application/json' } });
};
