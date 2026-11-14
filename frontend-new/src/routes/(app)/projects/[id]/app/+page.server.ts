import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

/** Where the backend's emails and notifications send people for a project: its workspace. */
export const load: PageServerLoad = ({ params }) => redirect(303, `/projects/${params.id}`);
