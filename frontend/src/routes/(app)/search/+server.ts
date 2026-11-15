import { error } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { apiFetch } from '$lib/server/api';

/** The app's search box (⌘K, $lib/search): the person's chats, memory and files. */
export const GET: RequestHandler = async ({ cookies, fetch, url }) => {
	const response = await apiFetch(fetch, cookies, `/api/search${url.search}`);
	if (!response.ok) throw error(response.status === 401 ? 401 : response.status === 400 ? 400 : 502, 'search unavailable');
	return new Response(await response.text(), { headers: { 'Content-Type': 'application/json', 'Cache-Control': 'no-store' } });
};
