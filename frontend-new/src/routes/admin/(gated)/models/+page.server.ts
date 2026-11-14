import { m } from '$lib/paraglide/messages';
import type { AdminModel } from '$lib/admin/api';
import { act, field, getJson, mustLoad, refuse, send } from '$lib/server/admin';
import { fail } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';

/**
 * Models (models: view): every chat model Nomi offers, which is the default for chats, which reads
 * files people's own models can't, and which builds projects. With write: turn one on or off, and
 * choose those three.
 */
export const load: PageServerLoad = async (event) => ({ models: mustLoad(await getJson<AdminModel[]>(event, '/api/admin/settings/llm/models')) });

export const actions: Actions = {
	enable: async (event) => {
		const form = await event.request.formData();
		const id = field(form, 'id');
		if (!id) return refuse('enable', m.admin_model_required());
		return act(event, 'enable', `/api/admin/settings/llm/models/${encodeURIComponent(id)}/enabled`, 'PUT', { enabled: form.get('enabled') === 'on' });
	},

	/** The three roles, each sent only when it changed; the first refusal stops the rest. */
	roles: async (event) => {
		const form = await event.request.formData();
		const changes: [string, 'PUT', unknown][] = [];
		const defaultId = field(form, 'default_id');
		if (defaultId && defaultId !== field(form, 'was_default')) changes.push([`/api/admin/settings/llm/models/${encodeURIComponent(defaultId)}/default`, 'PUT', undefined]);
		for (const role of ['files', 'coding'] as const) {
			const id = field(form, `${role}_id`);
			if (id !== field(form, `was_${role}`)) changes.push([`/api/admin/settings/llm/${role}-model`, 'PUT', { id: id || null }]);
		}
		for (const [path, method, body] of changes) {
			const result = await send(event, path, method, body);
			if (!result.ok) return fail(result.problem.status >= 400 ? result.problem.status : 500, { action: 'roles', ok: false as const, problem: result.problem });
		}
		return { action: 'roles', ok: true as const, data: null };
	},
};
