import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ params }) => {
	const rawUsername = params.username.replace(/^@/, '');
	throw redirect(307, `/@${rawUsername}`);
};
