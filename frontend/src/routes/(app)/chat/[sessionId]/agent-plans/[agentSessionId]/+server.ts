import { json } from '@sveltejs/kit';
import { fetchAgentPlans } from '$lib/server/fetchAgentPlans';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	const plans = await fetchAgentPlans(fetch, cookies, params.sessionId, params.agentSessionId);
	return json({ plans });
};
