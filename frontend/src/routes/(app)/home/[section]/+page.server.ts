import { error } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { HOME_SECTIONS, isHomeSection, type OutItem, type PlanItem, type TodayItem } from '$lib/home';
import type { HomeSectionPage } from '$lib/types';
import type { PageServerLoad } from './$types';

const PER_PAGE = 20;

export const load: PageServerLoad = async ({ params, url, cookies, fetch }) => {
	const section = params.section;
	if (!isHomeSection(section)) error(404, 'Not found');
	const page = Math.max(1, Number.parseInt(url.searchParams.get('page') ?? '1', 10) || 1);
	const response = await apiFetch(fetch, cookies, `/api/home/${section}?page=${page}&per_page=${PER_PAGE}`);
	const result = response.ok ? ((await response.json()) as HomeSectionPage<OutItem | TodayItem | PlanItem>) : null;
	return { section, ...HOME_SECTIONS[section], result, page, perPage: PER_PAGE };
};
