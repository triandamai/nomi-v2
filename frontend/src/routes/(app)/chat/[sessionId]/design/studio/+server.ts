import { error, json } from '@sveltejs/kit';
import { loadStudio } from '$lib/server/designStudio';
import { m } from '$lib/paraglide/messages';
import type { RequestHandler } from './$types';

// What the chat's Design Studio holds now: a project page shows it in its Design tab and reloads it
// when Sora finishes a turn, without leaving the project.
export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const { status, studio } = await loadStudio(fetch, cookies, params.sessionId);
	if (!studio) throw error(status, m.studio_not_found());
	return json(studio);
};
