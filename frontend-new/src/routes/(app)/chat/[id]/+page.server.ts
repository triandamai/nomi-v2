import { error } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import * as chat from '$lib/server/chat';
import type { PageServerLoad } from './$types';

/**
 * A chat: its newest page of messages (rendered), its name, pin and project (from the chats list:
 * there's no single-chat endpoint), the projects it can move to, and its model and thinking
 * level. A chat that's gone, or someone else's, is a 404 (the backend answers both alike).
 * `invalidate('chat:page')` reloads it (the live feed reconnected: nothing missed is replayed).
 */
export const load: PageServerLoad = async (event) => {
	event.depends('chat:page');
	const id = event.params.id;
	const [page, session, moveTargets, thinking, models, model] = await Promise.all([
		chat.messagePage(event, id),
		chat.findSession(event, id),
		chat.moveTargets(event),
		chat.thinkingLevel(event, id),
		chat.modelOptions(event),
		chat.sessionModel(event, id),
	]);
	if (!page.messages) {
		if (page.status === 404 || page.status === 403) error(404, { message: m.chat_missing_title() });
		error(page.status >= 500 ? 503 : page.status, { message: m.error_title() });
	}
	const title = session?.title?.trim() || m.recents_untitled();
	const project = moveTargets.find((target) => target.id === session?.in_project_id);
	const meta = project?.name ?? `chat_${id.replace(/-/g, '').slice(0, 6)}`;
	return {
		// The app shell's top bar (the layout draws the chat's own).
		title,
		meta,
		chat: { id, title, meta, pinned: session?.pinned ?? false, inProjectId: session?.in_project_id ?? null },
		messages: page.messages,
		hasOlder: page.hasOlder,
		projects: moveTargets,
		thinking,
		models,
		model,
	};
};
