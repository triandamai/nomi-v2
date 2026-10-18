import { error, json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { waitingIn } from '$lib/chatActivity';
import type { MessageItem } from '$lib/types';
import type { RequestHandler } from './$types';

// Where a chat's work stands, for a chat the person left while the crew was on it
// ($lib/backgroundChats): is a crew member still at it, has the crew answered since `since`,
// and is anything waiting on the person (a question, an approval, an account to connect)?

/** Delegation states that mean a crew member still has the work. */
const ACTIVE = new Set(['pending', 'claimed', 'processing']);

export const GET: RequestHandler = async ({ params, url, cookies, fetch }) => {
	const since = Date.parse(url.searchParams.get('since') ?? '') || 0;
	const [activityResponse, messagesResponse] = await Promise.all([
		apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/agent-activity`),
		apiFetch(fetch, cookies, `/api/sessions/${params.sessionId}/messages?limit=20`),
	]);
	if (!activityResponse.ok || !messagesResponse.ok) error(activityResponse.ok ? messagesResponse.status : activityResponse.status);

	const delegations = (await activityResponse.json()) as { status: string }[];
	const { messages } = (await messagesResponse.json()) as { messages: MessageItem[] };
	const recent = messages.filter((message) => Date.parse(message.created_at) >= since);
	const lastAsk = recent.findLastIndex((message) => message.sender === 'user');
	const answers = recent.slice(lastAsk + 1).filter((message) => message.sender === 'assistant');

	return json({
		busy: delegations.some((delegation) => ACTIVE.has(delegation.status)),
		replied: answers.length > 0,
		waiting: waitingIn(recent),
		agent: answers.at(-1)?.agent_display_name ?? recent.findLast((message) => message.sender === 'assistant')?.agent_display_name ?? null,
	});
};
