import type { RequestHandler } from './$types';
import { relayFile } from '$lib/server/attachments';

export const GET: RequestHandler = (event) => relayFile(event, `/api/attachments/${event.params.id}/content`);
