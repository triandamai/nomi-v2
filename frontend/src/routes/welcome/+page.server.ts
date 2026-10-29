import { apiUrl } from '$lib/server/api';
import type { ShowcaseApp } from '../showcase/+page.server';
import type { PageServerLoad } from './$types';

// How many showcase apps the landing page shows; the rest are on /showcase.
const SHOWCASE_LIMIT = 3;

// Public: what someone sees the first time they open Nomi (the app sends signed-out visitors
// at / here). The showcase strip is best-effort and simply doesn't show if it can't load.
export const load: PageServerLoad = async ({ fetch, locals }) => {
	const response = await fetch(apiUrl('/api/showcase')).catch(() => null);
	const apps: ShowcaseApp[] = response?.ok ? await response.json() : [];
	return { apps: apps.slice(0, SHOWCASE_LIMIT), signedIn: Boolean(locals.accessToken) };
};
