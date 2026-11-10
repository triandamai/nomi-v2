import { APP_VERSION } from '$lib/version';
import { getMaintenance } from '$lib/server/maintenance';
import type { LayoutServerLoad } from './$types';

// Every sign-in screen ends with this build's version and whether Nomi is up: the status line says
// what's true, so it reads "under maintenance" while the backend says so.
export const load: LayoutServerLoad = async ({ fetch }) => ({
	version: APP_VERSION,
	year: new Date().getFullYear(),
	healthy: !(await getMaintenance(fetch)),
});
