import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

// The voices and models the saved ElevenLabs key can use.
export const GET: RequestHandler = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/tools/speak/voices');
	if (!response.ok) return json({ error: (await response.text()) || null }, { status: response.status });
	return json(await response.json());
};
