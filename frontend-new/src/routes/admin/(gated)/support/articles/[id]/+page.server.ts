import { error, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { Article } from '$lib/admin/api';
import { articleFromForm } from '$lib/admin/support';
import { act, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * A help article to write (`/admin/support/articles/new`) or change (support: write; deleting needs
 * manage). There's no single-article endpoint: an existing one comes from the list.
 */
export const load: PageServerLoad = async (event) => {
	if (event.params.id === 'new') return { title: m.admin_article_new(), article: null };
	const list = mustLoad(await getJson<Article[]>(event, '/api/admin/support/articles'));
	if (!list.ok) return { title: m.admin_page_support(), article: list };
	const article = list.data.find((one) => one.id === event.params.id);
	if (!article) error(404, { message: m.admin_article_missing() });
	return { title: article.title, article: { ok: true as const, data: article } };
};

export const actions: Actions = {
	save: async (event) => {
		const read = articleFromForm(await event.request.formData());
		if (!read.ok) return refuse('save', read.message);
		if (event.params.id !== 'new') return act<Article>(event, 'save', `/api/admin/support/articles/${encodeURIComponent(event.params.id)}`, 'PUT', read.body);
		const created = await act(event, 'save', '/api/admin/support/articles', 'POST', read.body);
		if ('ok' in created && created.ok) redirect(303, '/admin/support');
		return created;
	},

	delete: async (event) => {
		const deleted = await act(event, 'delete', `/api/admin/support/articles/${encodeURIComponent(event.params.id)}`, 'DELETE');
		if ('ok' in deleted && deleted.ok) redirect(303, '/admin/support');
		return deleted;
	},
};
