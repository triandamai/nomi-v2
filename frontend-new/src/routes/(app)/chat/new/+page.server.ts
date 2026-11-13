import { m } from '$lib/paraglide/messages';
import * as chat from '$lib/server/chat';
import type { PageServerLoad } from './$types';

/** A blank chat (New chat, ⌘K): the models its first message can go out on. */
export const load: PageServerLoad = async (event) => ({ title: m.nav_new_chat(), models: await chat.modelOptions(event) });
