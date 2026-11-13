import { chatRequest } from '$lib/server/chatRoutes';
import type { RequestHandler } from './$types';

// The chat page's own calls (src/lib/server/chatRoutes.ts): messages, the chat's settings,
// feedback, running a turn again, retrying a failed run, sharing.
const handle: RequestHandler = (event) => chatRequest(event, event.params.id, event.params.path.split('/').filter(Boolean), event.request);

export const GET = handle;
export const POST = handle;
export const PUT = handle;
export const PATCH = handle;
export const DELETE = handle;
