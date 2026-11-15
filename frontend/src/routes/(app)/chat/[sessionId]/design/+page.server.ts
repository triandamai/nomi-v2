import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { StudioResponse } from '$lib/designStudio';
import type { PageServerLoad } from './$types';

// The Design Studio for one chat: its preview versions, what was said about them, and the one the link asked for.
export const load: PageServerLoad = async ({ params, cookies, fetch, url }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/design`);
	if (!response.ok) throw error(response.status, m.studio_not_found());
	const { previews, feedback } = (await response.json()) as Partial<StudioResponse>;
	return { previews: previews ?? [], feedback: feedback ?? [], wanted: url.searchParams.get('version'), sessionId: params.sessionId };
};
