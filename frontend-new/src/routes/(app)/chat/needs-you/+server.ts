import { json } from '@sveltejs/kit';
import { homeSummary } from '$lib/server/chat';
import { needsYou } from '$lib/chat/needsYou';
import type { RequestHandler } from './$types';

/**
 * What waits on the person in their chats (Home's needs-you items: approvals paused for them), for
 * a phone to say so while they're elsewhere in the app (UX contract phone-chat-flows, S3.5 interim).
 */
export const GET: RequestHandler = async (event) => json(needsYou(await homeSummary(event)));
