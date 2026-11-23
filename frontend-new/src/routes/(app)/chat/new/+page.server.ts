import { m } from '$lib/paraglide/messages';
import * as chat from '$lib/server/chat';
import type { PageServerLoad } from './$types';

/**
 * A blank chat (New chat, ⌘K): the models its first message can go out on, and `?q=`, a message to
 * start from (the landing page's composer, carried through sign-in), put in the field unsent.
 */
export const load: PageServerLoad = async (event) => ({
	title: m.nav_new_chat(),
	models: await chat.modelOptions(event),
	draft: (event.url.searchParams.get('q') ?? '').slice(0, 2000),
});
