import { error, fail } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadChatPage } from '$lib/server/chatPage';
import { changeDomain, loadDeploy, loadDomain, loadProject, publish, rollBack } from '$lib/server/workspace';
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
	/** Publishes an earlier deploy again, as a new one. */
	rollback: async (event) => {
		const id = String((await event.request.formData()).get('publish') ?? '');
		if (!id) return fail(400, { message: m.action_failed() });
		const sent = await rollBack(event, event.params.id, id);
		return sent.ok ? { rolledBack: id } : refused(sent);
	},
};
