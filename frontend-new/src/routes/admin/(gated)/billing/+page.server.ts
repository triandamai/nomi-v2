import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

/** Billing is its three pages: the console opens on terms (the old app's /admin/billing). */
export const load: PageServerLoad = () => redirect(307, '/admin/billing/terms');
