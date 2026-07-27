import { fail } from '@sveltejs/kit';
import type { Actions } from './$types';

export const actions: Actions = {
	newChat: async () => {
		return fail(501, { error: 'not implemented yet' });
	},
};
