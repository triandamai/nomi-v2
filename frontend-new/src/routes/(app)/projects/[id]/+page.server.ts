import { error } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import { loadChatPage } from '$lib/server/chatPage';
import { loadProject } from '$lib/server/workspace';
import type { PageServerLoad } from './$types';

/** A project's workspace (Flow / Build): the project, where it's live, and its chat. */
export const load: PageServerLoad = async (event) => {
	event.depends('app:workspace-project');
	const loaded = await loadProject(event, event.params.id);
	if (!loaded.project) error(loaded.status === 404 || loaded.status === 403 ? 404 : 503, { message: loaded.status === 404 || loaded.status === 403 ? m.workspace_project_missing() : m.error_title() });
	const chat = await loadChatPage(event, loaded.project.session_id);
	return { ...chat, title: loaded.project.name, project: loaded.project, live: loaded.live, domain: loaded.domain };
};
