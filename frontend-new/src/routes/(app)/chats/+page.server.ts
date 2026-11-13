import { error } from '@sveltejs/kit';
import { m } from '$lib/paraglide/messages';
import * as chat from '$lib/server/chat';
import type { PageServerLoad } from './$types';

/**
 * The Chats page: every chat of the person's (the projects' own live in their projects). It
 * reloads with the shell (`app:shell`), so a chat pinned, renamed or deleted anywhere shows here.
 */
export const load: PageServerLoad = async (event) => {
	event.depends('app:shell');
	const sessions = await chat.chatList(event);
	if (!sessions) error(503, { message: m.error_title() });
	const count = sessions.filter((session) => !session.project_id).length;
	return { title: m.chats_title(), meta: m.chats_meta({ count }), sessions };
};
