import { startRequest } from '$lib/server/chatRoutes';
import type { RequestHandler } from './$types';

/** A new chat with its first message (Home's composer, and /chat/new). */
export const POST: RequestHandler = (event) => startRequest(event, event.request);
