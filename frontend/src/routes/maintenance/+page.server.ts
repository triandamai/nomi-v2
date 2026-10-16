import { maintenanceState } from '$lib/server/maintenance';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ fetch }) => ({ maintenance: await maintenanceState(fetch) });
