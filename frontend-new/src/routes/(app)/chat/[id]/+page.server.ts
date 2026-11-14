import { loadChatPage } from '$lib/server/chatPage';
import type { PageServerLoad } from './$types';

/** A chat (`invalidate('chat:page')` reloads it: the live feed reconnected, nothing missed is replayed). */
export const load: PageServerLoad = (event) => loadChatPage(event, event.params.id);
