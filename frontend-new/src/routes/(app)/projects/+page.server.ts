import { fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { deleteProject, duplicateProject, loadProjects, renameProject, startProject, tellKoda } from '$lib/server/projects';
import { publish, transfer } from '$lib/server/workspace';
import type { Actions, PageServerLoad, RequestEvent } from './$types';

/**
 * Projects (App / Projects JyWON): the projects of the workspace they come from, which of their
 * chats Koda is at work in, and whether this Nomi publishes. When the list can't be read the page
 * says so with the status (Loading, offline & errors c6fFB, step 2) and the composer still works.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:projects');
	// The shell picks a workspace for projects when none is chosen yet: list after it has.
	await event.parent();
	const { list, status, members, chats, publishing } = await loadProjects(event);
	if (!list) return { title: m.nav_projects(), list: null, failure: { status: status ?? 503, at: new Date().toISOString() }, members, chats, publishing };
	const count = list.projects.length;
	return { title: m.nav_projects(), meta: count === 1 ? m.projects_meta_one() : m.projects_meta({ count }), list, failure: null, members, chats, publishing };
};

const field = async (event: RequestEvent, ...names: string[]) => {
	const form = await event.request.formData();
	return names.map((name) => String(form.get(name) ?? '').trim());
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
		const [id] = await field(event, 'id');
		if (!id) return fail(400, { message: m.action_failed() });
		const deleted = await deleteProject(event, id);
		return deleted.ok ? { deleted: id } : fail(deleted.status, { message: deleted.message || m.action_failed() });
	},
	/** Stop on a building card: the project's chat is told to stop, as the chat's own Stop does. */
	stop: async (event) => {
		const [session] = await field(event, 'session');
		if (!session) return fail(400, { message: m.action_failed() });
		const sent = await tellKoda(event, session, 'stop');
		return sent.ok ? { stopped: session } : fail(sent.status, { message: sent.message || m.action_failed() });
	},
	/** Resume on a stopped card: Koda is asked to carry on where it left the build. */
	resume: async (event) => {
		const [session] = await field(event, 'session');
		if (!session) return fail(400, { message: m.action_failed() });
		const sent = await tellKoda(event, session, m.projects_resume_text());
		return sent.ok ? { resumed: session } : fail(sent.status, { message: sent.message || m.action_failed() });
	},
	/** Ask Koda to fix, on a card whose last deploy failed: said in the project's chat, where Koda works. */
	fix: async (event) => {
		const [session] = await field(event, 'session');
		if (!session) return fail(400, { message: m.action_failed() });
		const sent = await tellKoda(event, session, m.projects_fix_text());
		return sent.ok ? { asked: session } : fail(sent.status, { message: sent.message || m.action_failed() });
	},
	/** Ask Koda to check, on a live card that's down. */
	check: async (event) => {
		const [session] = await field(event, 'session');
		if (!session) return fail(400, { message: m.action_failed() });
		const sent = await tellKoda(event, session, m.projects_check_text());
		return sent.ok ? { asked: session } : fail(sent.status, { message: sent.message || m.action_failed() });
	},
	/** Rename from the card (F2 or its menu): the new name, kept by the backend for its builders. */
	rename: async (event) => {
		const [id, name] = await field(event, 'id', 'name');
		if (!id || !name) return fail(400, { message: m.projects_name_needed() });
		const sent = await renameProject(event, id, name);
		return sent.ok ? { renamed: id } : fail(sent.status, { message: sent.message || m.action_failed() });
	},
	/** Duplicate from the card (⌘D or its menu): a copy in the same workspace. */
	duplicate: async (event) => {
		const [id] = await field(event, 'id');
		if (!id) return fail(400, { message: m.action_failed() });
		const made = await duplicateProject(event, id);
		return made.ok ? { duplicated: made.projectId } : fail(made.status, { message: made.message || m.action_failed() });
	},
	/** Asked from the detail panel: the change goes to Koda in the project's chat. */
	ask: async (event) => {
		const [session, text] = await field(event, 'session', 'text');
		if (!session || !text) return fail(400, { message: m.projects_describe() });
		const sent = await tellKoda(event, session, text);
		return sent.ok ? { asked: session } : fail(sent.status, { message: sent.message || m.action_failed() });
	},
	/** Deploy on a draft card: its first publish; the card follows it from there. */
	deploy: async (event) => {
		const [id] = await field(event, 'id');
		if (!id) return fail(400, { message: m.action_failed() });
		const sent = await publish(event, id);
		return sent.ok ? { deploying: id } : fail(sent.status, { message: sent.message || m.deploy_couldnt_start() });
	},
	/** Move from the card menu: the project goes to the workspace picked in the Move dialog, which is the confirmation. */
	move: async (event) => {
		const [id, target, name] = await field(event, 'id', 'target', 'name');
		if (!id || !target || !name) return fail(400, { message: m.action_failed() });
		const moved = await transfer(event, id, target, name);
		return moved.ok ? { moved: id } : fail(moved.status, { message: moved.message || m.action_failed() });
	},
};
