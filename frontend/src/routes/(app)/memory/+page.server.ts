import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { MemoryListResponse } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/memory');
	const { memories } = response.ok ? ((await response.json()) as MemoryListResponse) : { memories: [] };
	return { memories, loadFailed: !response.ok };
};

export const actions: Actions = {
	forget: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		if (typeof id !== 'string' || !id) {
			return fail(400, { error: m.mem_pick() });
		}
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!response.ok && response.status !== 404) {
			return fail(response.status, { error: m.mem_forget_failed() });
		}
		return { forgotten: id };
	},
	/** The person rewrote a memory (and maybe its kind). */
	edit: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const id = data.get('id');
		const content = String(data.get('content') ?? '').trim();
		const kind = data.get('kind');
		if (typeof id !== 'string' || !id) return fail(400, { error: m.mem_pick() });
		if (!content) return fail(400, { error: m.mem_edit_empty() });
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}`, {
			method: 'PUT',
			body: JSON.stringify({ content, kind: typeof kind === 'string' && kind ? kind : null }),
		});
		if (!response.ok) return fail(response.status, { error: m.mem_edit_failed() });
		return { edited: id };
	},
	/** The person says a memory is still true. */
	confirm: async ({ request, cookies, fetch }) => {
		const id = (await request.formData()).get('id');
		if (typeof id !== 'string' || !id) return fail(400, { error: m.mem_pick() });
		const response = await apiFetch(fetch, cookies, `/api/memory/${encodeURIComponent(id)}/confirm`, { method: 'POST' });
		if (!response.ok && response.status !== 404) return fail(response.status, { error: m.mem_confirm_failed() });
		return { confirmed: id };
	},
};
