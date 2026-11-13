import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import type { OrgList } from '$lib/adapters/api';
import { apiFetch } from '$lib/server/api';
import { chooseProjectsWorkspace, createWorkspace } from '$lib/server/account';
import type { Actions, PageServerLoad } from './$types';

/** Settings › Workspaces (App / Settings / Workspaces l222Mz): the person's workspaces. */
export const load: PageServerLoad = async (event) => {
	event.depends('app:workspaces');
	const response = await apiFetch(event, '/api/orgs').catch(() => null);
	if (!response?.ok) error(503, { message: m.workspaces_load_failed() });
	const list = (await response.json()) as OrgList;
	const count = list.orgs.length;
	return { title: m.settings_workspaces(), meta: count === 1 ? m.workspaces_meta_one() : m.workspaces_meta({ count }), list };
};

export const actions: Actions = {
	create: async (event) => {
		const made = await createWorkspace(event, String((await event.request.formData()).get('name') ?? ''));
		if (made.ok) return { created: made.org };
		const message = made.field === 'required' ? m.workspace_name_required() : made.field === 'long' ? m.workspace_name_long() : made.message || m.workspace_create_failed();
		return fail(made.status, { field: 'name', message });
	},
	/** Where projects come from (the backend's one choice of workspace). */
	use: async (event) => {
		const id = String((await event.request.formData()).get('id') ?? '');
		if (!id) return fail(400, { message: m.workspace_choose_failed() });
		const chosen = await chooseProjectsWorkspace(event, id);
		return chosen.ok ? { used: id } : fail(chosen.status, { message: chosen.message || m.workspace_choose_failed() });
	},
};
