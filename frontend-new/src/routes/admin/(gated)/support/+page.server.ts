import { m } from '$lib/paraglide/messages';
import type { Article, AskedQuestion, Draft, SupportKey } from '$lib/admin/api';
import { articleFromForm } from '$lib/admin/support';
import { act, field, getJson, mustLoad, refuse } from '$lib/server/admin';
import type { Actions, PageServerLoad } from './$types';

/**
 * Support (support: view): what the help desk answers from. The drafts Koda wrote from questions it
 * couldn't answer (write: publish with edits, or dismiss), the questions asked, the help articles
 * (`?q=` searches them), and the keys products ask with (manage: make one, shown once, or revoke).
 */
export const load: PageServerLoad = async (event) => {
	const query = (event.url.searchParams.get('q') ?? '').trim();
	const [drafts, questions, keys, articles] = await Promise.all([
		getJson<Draft[]>(event, '/api/admin/support/drafts?status=open'),
		getJson<AskedQuestion[]>(event, '/api/admin/support/questions'),
		getJson<SupportKey[]>(event, '/api/admin/support/keys'),
		getJson<Article[]>(event, `/api/admin/support/articles${query ? `?q=${encodeURIComponent(query)}` : ''}`),
	]);
	return { query, drafts: mustLoad(drafts), questions, keys, articles };
};

export const actions: Actions = {
	publishDraft: async (event) => {
		const form = await event.request.formData();
		const id = field(form, 'id');
		const read = articleFromForm(form, { published: true });
		if (!id) return refuse('publishDraft', m.admin_draft_required());
		if (!read.ok) return refuse('publishDraft', read.message);
		return act<Article>(event, 'publishDraft', `/api/admin/support/drafts/${encodeURIComponent(id)}/publish`, 'POST', read.body);
	},

	dismissDraft: async (event) => {
		const id = field(await event.request.formData(), 'id');
		if (!id) return refuse('dismissDraft', m.admin_draft_required());
		return act(event, 'dismissDraft', `/api/admin/support/drafts/${encodeURIComponent(id)}/dismiss`, 'POST');
	},

	createKey: async (event) => {
		const form = await event.request.formData();
		const name = field(form, 'name');
		if (!name) return refuse('createKey', m.admin_key_name_required());
		// The new key comes back with its secret beside its fields, shown this once.
		return act<SupportKey & { secret: string }>(event, 'createKey', '/api/admin/support/keys', 'POST', { name, product_name: field(form, 'product_name') });
	},

	revokeKey: async (event) => {
		const id = field(await event.request.formData(), 'id');
		if (!id) return refuse('revokeKey', m.admin_key_required());
		return act(event, 'revokeKey', `/api/admin/support/keys/${encodeURIComponent(id)}`, 'DELETE');
	},
};
