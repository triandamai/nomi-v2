import { error, json } from '@sveltejs/kit';
import { fetchMessagePage } from '$lib/server/fetchMessagePage';
import type { RequestHandler } from './$types';

/** Earlier messages for "load earlier" in the chat: the page before `?before=<message id>`. */
export const GET: RequestHandler = async ({ params, url, cookies, fetch }) => {
	const before = url.searchParams.get('before');
	if (!before) throw error(400, 'before is required');
	const page = await fetchMessagePage(fetch, cookies, params.sessionId, before);
	if (!page.messages) throw error(page.status, 'could not load messages');
	return json({ messages: page.messages, hasOlder: page.hasOlder });
};
