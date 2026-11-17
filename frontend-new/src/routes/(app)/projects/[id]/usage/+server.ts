import { isRedirect, json } from '@sveltejs/kit';
import type { ProjectUsage } from '$lib/adapters/api';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

const MONTH = /^\d{4}-\d{2}$/;
/** What's said when the backend couldn't be reached. */
const UNREACHABLE = 503;

/** The Tokens & cost sheet's month (`?month=YYYY-MM`, this month without one). */
export const GET: RequestHandler = async (event) => {
	const month = event.url.searchParams.get('month');
	const query = month && MONTH.test(month) ? `?month=${month}` : '';
	let response: Response;
	try {
		response = await apiFetch(event, `/api/projects/${encodeURIComponent(event.params.id)}/usage${query}`);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return json({ error: 'unreachable' }, { status: UNREACHABLE });
	}
	if (!response.ok) return json({ error: (await response.text()).trim().slice(0, 300) }, { status: response.status });
	return json((await response.json()) as ProjectUsage);
};
