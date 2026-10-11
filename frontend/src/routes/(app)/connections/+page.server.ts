import { apiFetch } from '$lib/server/api';
import type { ConnectionsOverview } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const response = await apiFetch(fetch, cookies, '/api/connections');
	return {
		overview: response.ok ? ((await response.json()) as ConnectionsOverview) : null,
		query: url.searchParams.get('q') ?? '',
	};
};
