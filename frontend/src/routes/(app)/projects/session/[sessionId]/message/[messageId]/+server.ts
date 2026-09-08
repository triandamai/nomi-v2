import { json } from '@sveltejs/kit';
import { fetchRenderedMessage } from '$lib/server/fetchRenderedMessage';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const message = await fetchRenderedMessage(fetch, cookies, params.sessionId, params.messageId);
	return json(message);
};
