import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { m } from '$lib/paraglide/messages';
import type { StudioResponse } from '$lib/designStudio';
import type { PageServerLoad } from './$types';

// The Design Studio for one chat: its preview versions, and the one the link asked for.
export const load: PageServerLoad = async ({ params, cookies, fetch, url }) => {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/design`);
	if (!response.ok) throw error(response.status, m.studio_not_found());
	const { previews } = (await response.json()) as StudioResponse;
	return { previews, wanted: url.searchParams.get('version'), sessionId: params.sessionId };
};
