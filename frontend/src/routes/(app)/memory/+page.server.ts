import { apiFetch } from '$lib/server/api';
import type { MemoryListResponse } from '$lib/types';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/memory');
	const { memories } = response.ok ? ((await response.json()) as MemoryListResponse) : { memories: [] };
	return { memories };
};
