import { error, fail, redirect } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadChatPage } from '$lib/server/chatPage';
import { changeDomain, loadDeploy, loadDomain, loadProject, publish, rollBack, takeDown, transfer } from '$lib/server/workspace';
import { deleteProject } from '$lib/server/projects';
import type { Actions, PageServerLoad } from './$types';

/** A project's workspace (Flow / Build): the project, where it's live, and its chat. */
export const load: PageServerLoad = async (event) => {
	event.depends('app:workspace-project');
	const loaded = await loadProject(event, event.params.id);
	if (!loaded.project) error(loaded.status === 404 || loaded.status === 403 ? 404 : 503, { message: loaded.status === 404 || loaded.status === 403 ? m.workspace_project_missing() : m.error_title() });
	const [chat, deploy, address] = await Promise.all([loadChatPage(event, loaded.project.session_id), loadDeploy(event, event.params.id), loadDomain(event, event.params.id)]);
	return { ...chat, title: loaded.project.name, project: loaded.project, live: loaded.live, domain: loaded.domain, deploy, address };
};

const refused = (sent: { status: number; message: string }) => fail(sent.status, { message: sent.message || m.action_failed() });

export const actions: Actions = {
	/** Sends what's in the preview live, as the next deploy; it runs on its own. */
	publish: async (event) => {
		const sent = await publish(event, event.params.id);
		return sent.ok ? { publishing: true } : refused(sent);
	},
	/** Moves the live app to another name under Nomi's apps domain. */
	domain: async (event) => {
		const name = String((await event.request.formData()).get('name') ?? '').trim().toLowerCase();
		if (!name) return fail(400, { message: m.domain_name_needed() });
		const changed = await changeDomain(event, event.params.id, name);
		if (changed.ok) return { url: changed.url };
		return fail(changed.status, { message: changed.message || m.action_failed(), suggestions: changed.answer?.suggestions ?? [] });
	},
	/** Gives the project to another workspace; the name, typed, confirms it. */
	transfer: async (event) => {
		const form = await event.request.formData();
		const [to, name] = [String(form.get('to') ?? ''), String(form.get('name') ?? '').trim()];
		if (!to) return fail(400, { message: m.transfer_pick() });
		const sent = await transfer(event, event.params.id, to, name);
		if (!sent.ok) return refused(sent);
		redirect(303, `/projects?moved=${encodeURIComponent(name)}`);
	},
	/** Stops serving the live app; publishing brings it back. */
	takedown: async (event) => {
		const sent = await takeDown(event, event.params.id);
		return sent.ok ? { takenDown: true } : refused(sent);
	},
	/** Deletes the project for good, once its name is typed; Projects says so. */
	delete: async (event) => {
		const name = String((await event.request.formData()).get('name') ?? '').trim();
		const loaded = await loadProject(event, event.params.id);
		if (!loaded.project) return fail(404, { message: m.workspace_project_missing() });
		if (name !== loaded.project.name) return fail(400, { field: 'name', message: m.delete_name_mismatch() });
		const deleted = await deleteProject(event, event.params.id);
		if (!deleted.ok) return refused(deleted);
		redirect(303, `/projects?deleted=${encodeURIComponent(name)}`);
	},
	/** Publishes an earlier deploy again, as a new one. */
	rollback: async (event) => {
		const id = String((await event.request.formData()).get('publish') ?? '');
		if (!id) return fail(400, { message: m.action_failed() });
		const sent = await rollBack(event, event.params.id, id);
		return sent.ok ? { rolledBack: id } : refused(sent);
	},
};
