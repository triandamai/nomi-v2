import type { EventFeed } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

/** The events Nomi received from Shipyard (integrations: view), newest first, a page at a time. */
export const load: PageServerLoad = async (event) => {
	const asked = Number.parseInt(event.url.searchParams.get('page') ?? '1', 10);
	const page = Number.isFinite(asked) && asked > 0 ? asked : 1;
	return { feed: mustLoad(await getJson<EventFeed>(event, `/api/admin/shipyard/events?page=${page}`)) };
};
