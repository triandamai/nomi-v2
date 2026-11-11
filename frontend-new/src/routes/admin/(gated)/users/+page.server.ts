import type { UserList } from '$lib/admin/api';
import { getJson, mustLoad } from '$lib/server/admin';
import type { PageServerLoad } from './$types';

const PAGE_SIZE = 20;

/** Everyone on Nomi (user: view), twenty a page, searched by email: `?query=` and `?page=`. */
export const load: PageServerLoad = async (event) => {
	const query = (event.url.searchParams.get('query') ?? '').trim();
	const asked = Number.parseInt(event.url.searchParams.get('page') ?? '1', 10);
	const page = Number.isFinite(asked) && asked > 0 ? asked : 1;
	const params = new URLSearchParams({ page: String(page), page_size: String(PAGE_SIZE) });
	if (query) params.set('query', query);
	return { list: mustLoad(await getJson<UserList>(event, `/api/admin/users?${params}`)), query, page, pageSize: PAGE_SIZE };
};
