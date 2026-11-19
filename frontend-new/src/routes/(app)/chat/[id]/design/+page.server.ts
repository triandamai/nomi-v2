import { error } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadStudio } from '$lib/server/designStudio';
import type { PageServerLoad } from './$types';

/** The Design Studio for one chat: its directions, preview versions, feedback, approval, and the version the link asked for. */
export const load: PageServerLoad = async (event) => {
	const { status, studio } = await loadStudio(event, event.params.id);
	if (!studio) error(status === 404 || status === 403 ? 404 : 503, { message: m.studio_not_found() });
	return {
		...studio,
		title: m.studio_title(),
		wanted: event.url.searchParams.get('version'),
		sessionId: event.params.id,
	};
};
