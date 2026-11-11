import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

/** Settings opens on Profile, the first of its pages (App / Settings / Profile U9Gnrg). */
export const load: PageServerLoad = () => redirect(307, '/settings/profile');
