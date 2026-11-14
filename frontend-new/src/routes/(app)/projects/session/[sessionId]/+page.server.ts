import { redirect } from '@sveltejs/kit';
import { projectOfChat } from '$lib/server/workspace';
import type { PageServerLoad } from './$types';

/** The old app's project address (by its chat): the project's workspace, or the chat itself. */
export const load: PageServerLoad = async (event) => {
	const id = await projectOfChat(event, event.params.sessionId);
	redirect(303, id ? `/projects/${id}` : `/chat/${event.params.sessionId}`);
};
