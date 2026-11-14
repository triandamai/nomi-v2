import { json } from '@sveltejs/kit';
import type { MetricsRange } from '$lib/adapters/api';
import { loadMonitor } from '$lib/server/workspace';
import type { RequestHandler } from './$types';

const RANGES: MetricsRange[] = ['1h', '24h', '7d'];

/** The Monitor sheet's numbers (`?range=1h|24h|7d`), asked for again every 30 seconds while it's open. */
export const GET: RequestHandler = async (event) => {
	const asked = event.url.searchParams.get('range') as MetricsRange;
	return json(await loadMonitor(event, event.params.id, RANGES.includes(asked) ? asked : '24h'));
};
