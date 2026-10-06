import { json } from '@sveltejs/kit';
import { fetchMessageMemories } from '$lib/server/fetchMessageMemories';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	return json(await fetchMessageMemories(fetch, cookies, params.sessionId, params.messageId));
};
