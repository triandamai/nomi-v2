import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { deleteProject, loadProjects, startProject } from '$lib/server/projects';
import type { Actions, PageServerLoad } from './$types';

/** Projects (App / Projects JyWON): the projects of the workspace they come from. */
export const load: PageServerLoad = async (event) => {
	event.depends('app:projects');
	// The shell picks a workspace for projects when none is chosen yet: list after it has.
	await event.parent();
	const { list, members } = await loadProjects(event);
	if (!list) error(503, { message: m.projects_load_failed() });
	const count = list.projects.length;
	return { title: m.nav_projects(), meta: count === 1 ? m.projects_meta_one() : m.projects_meta({ count }), list, members };
};

export const actions: Actions = {
	/** A new project from the composer; it opens in its workspace, Koda already on it. */
	start: async (event) => {
		const form = await event.request.formData();
		const description = String(form.get('description') ?? '').trim();
		if (!description) return fail(400, { message: m.projects_describe() });
		const org = String(form.get('org') ?? '') || null;
		const started = await startProject(event, org, description);
		if (!started.ok) return fail(started.status, { needsWorkspace: started.needsWorkspace ?? false, message: started.needsWorkspace ? m.projects_needs_workspace() : started.message || m.projects_start_failed() });
		redirect(303, `/projects/${started.projectId}`);
	},
	delete: async (event) => {
		const id = String((await event.request.formData()).get('id') ?? '');
		if (!id) return fail(400, { message: m.action_failed() });
		const deleted = await deleteProject(event, id);
		return deleted.ok ? { deleted: id } : fail(deleted.status, { message: deleted.message || m.action_failed() });
	},
};
