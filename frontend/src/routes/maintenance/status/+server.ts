import { json } from '@sveltejs/kit';
import { maintenanceState } from '$lib/server/maintenance';
import type { RequestHandler } from './$types';

// Polled by the maintenance page (and the app) to notice when maintenance starts or ends.
export const GET: RequestHandler = async ({ fetch }) => {
	return json(await maintenanceState(fetch), { headers: { 'cache-control': 'no-store' } });
};
