import { error } from '@sveltejs/kit';
import { loadPlan } from '$lib/server/plan';
import { usageCsv } from '$lib/settings/plan';
import type { RequestHandler } from './$types';

/** Plan & usage's "Export usage CSV": the month shown (`?month=`), as a file to keep. */
export const GET: RequestHandler = async (event) => {
	const data = await loadPlan(event, event.url.searchParams.get('month'));
	if (!data) error(503, 'usage unavailable');
	return new Response(usageCsv(data.usage), {
		headers: { 'content-type': 'text/csv; charset=utf-8', 'content-disposition': `attachment; filename="nomi-usage-${data.usage.month}.csv"` },
	});
};
