import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// The voices and models the saved ElevenLabs key can use.
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/tools/speak/voices');
	// Errors come as `{ error: <code>, detail? }`; the sheet words them in the admin's language.
	if (!response.ok) return json(await response.json().catch(() => ({ error: null })), { status: response.status });
	return json(await response.json());
};
