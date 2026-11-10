import { redirect } from '@sveltejs/kit';
import { getMaintenance } from '$lib/server/maintenance';
import type { PageServerLoad } from './$types';

/** While Nomi is down for maintenance, what the admins said and until when; once it's back, home. */
export const load: PageServerLoad = async ({ fetch }) => {
	const maintenance = await getMaintenance(fetch);
	if (!maintenance) redirect(307, '/');
	return maintenance;
};
