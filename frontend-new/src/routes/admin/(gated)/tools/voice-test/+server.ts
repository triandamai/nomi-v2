import { isRedirect, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { readApiError } from '$lib/server/apiError';
import type { RequestHandler } from './$types';

/**
 * Tools → voice → test (tools: write, checked by the backend): a line said with the saved voice,
 * handed back as the recording for the page to play. A refusal comes back with the service's words.
 */
export const POST: RequestHandler = async (event) => {
	const body = (await event.request.json().catch(() => ({}))) as { text?: unknown };
	let response: Response;
	try {
		response = await apiFetch(event, '/api/admin/tools/speak/test', { method: 'POST', body: JSON.stringify({ text: typeof body.text === 'string' ? body.text : null }) });
	} catch (error) {
		if (isRedirect(error)) return json({ error: 'signed_out' }, { status: 401 });
		return json({ error: '' }, { status: 503 });
	}
	if (!response.ok) return json({ error: (await readApiError(response)).message }, { status: response.status });
	return new Response(response.body, { headers: { 'content-type': response.headers.get('content-type') ?? 'audio/mpeg' } });
};
