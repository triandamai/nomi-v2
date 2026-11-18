import { error } from '@sveltejs/kit';
import { loadStudio } from '$lib/server/designStudio';
import { m } from '$lib/paraglide/messages';
import type { PageServerLoad } from './$types';

// The Design Studio for one chat: its preview versions, what was said about them, and the one the link asked for.
export const load: PageServerLoad = async ({ params, cookies, fetch, url }) => {
	const { status, studio } = await loadStudio(fetch, cookies, params.sessionId);
	if (!studio) throw error(status, m.studio_not_found());
	return { ...studio, wanted: url.searchParams.get('version'), sessionId: params.sessionId };
};
