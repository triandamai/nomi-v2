import * as chat from '$lib/server/chat';
import type { PageServerLoad } from './$types';

/** Home: what the Today strip sums up, and the models a new chat can start on. */
export const load: PageServerLoad = async (event) => {
	const [home, models] = await Promise.all([chat.homeSummary(event), chat.modelOptions(event)]);
	return { home, models, loadedAt: new Date().toISOString() };
};
