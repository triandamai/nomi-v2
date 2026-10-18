import { error, json } from '@sveltejs/kit';
import { loadCatalog } from '$lib/server/providers';
import type { RequestHandler } from './$types';

// The provider catalog for sheets that connect one in place (the chat page's model sheet),
// fetched when they open rather than with every chat.
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const providers = await loadCatalog(fetch, cookies);
	if (!providers) throw error(502, 'Providers unavailable');
	return json({ providers }, { headers: { 'cache-control': 'private, max-age=300' } });
};
