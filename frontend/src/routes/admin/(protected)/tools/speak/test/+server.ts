import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// Says a line with the saved voice settings and hands back the recording.
export const POST: RequestHandler = async ({ request, cookies, fetch }) => {
	const body = (await request.json().catch(() => ({}))) as { text?: string };
	const response = await apiFetch(fetch, cookies, '/api/admin/tools/speak/test', {
		method: 'POST',
		body: JSON.stringify({ text: body.text ?? null }),
	});
	// Errors come as `{ error: <code>, detail? }`; the sheet words them in the admin's language.
	if (!response.ok) return json(await response.json().catch(() => ({ error: null })), { status: response.status });
	return new Response(response.body, { headers: { 'content-type': 'audio/mpeg', 'cache-control': 'no-store' } });
};
