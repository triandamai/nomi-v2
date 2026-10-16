import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// Shipyard lives under Integrations now.
export const load: PageServerLoad = () => {
	throw redirect(308, '/admin/integrations');
};
