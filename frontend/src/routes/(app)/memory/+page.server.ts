import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { MemoryListResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/memory');
	const { memories } = response.ok ? ((await response.json()) as MemoryListResponse) : { memories: [] };
	return { memories, loadFailed: !response.ok };
};

export const actions: Actions = {
	forget: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		if (typeof id !== 'string' || !id) {
			return fail(400, { error: 'Pick a memory to forget.' });
		}
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!response.ok && response.status !== 404) {
			return fail(response.status, { error: 'Nomi couldn’t forget that. Try again.' });
		}
		return { forgotten: id };
	},
};
