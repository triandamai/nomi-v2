import { apiUrl } from '$lib/server/api';
import type { PageServerLoad } from './$types';

export interface ShowcaseApp {
	id: string;
	name: string;
	blurb: string | null;
	url: string;
	stack: string;
	team: string | null;
	showcased_at: string;
}

// Public: anyone can browse what people built, signed in or not.
export const load: PageServerLoad = async ({ fetch, locals }) => {
	const response = await fetch(apiUrl('/api/showcase')).catch(() => null);
	const apps: ShowcaseApp[] = response?.ok ? await response.json() : [];
	return { apps, signedIn: Boolean(locals.accessToken), failed: !response?.ok };
};
