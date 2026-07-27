import type { PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies }) => {
	return { userEmail: cookies.get('user_email') ?? '' };
};
