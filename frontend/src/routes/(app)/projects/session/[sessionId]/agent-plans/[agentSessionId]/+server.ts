import { error, json } from '@sveltejs/kit';
import { fetchAgentPlans } from '$lib/server/fetchAgentPlans';
import { setAgentPlanItem } from '$lib/server/setAgentPlanItem';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, cookies, fetch }) => {
	return json(await fetchAgentPlans(fetch, cookies, params.sessionId, params.agentSessionId));
};

/** Ticks a plan step; answers with the refreshed versions (and build) so the bubble and sheet stay in step. */
export const PATCH: RequestHandler = async ({ params, cookies, fetch, request }) => {
	const body = (await request.json().catch(() => null)) as { planId?: unknown; index?: unknown; done?: unknown } | null;
	if (!body || typeof body.planId !== 'string' || !Number.isInteger(body.index) || (body.index as number) < 0 || typeof body.done !== 'boolean') {
		throw error(400, 'planId, index and done are required');
	}
	await setAgentPlanItem(fetch, cookies, params.sessionId, params.agentSessionId, body.planId, body.index as number, body.done);
	return json(await fetchAgentPlans(fetch, cookies, params.sessionId, params.agentSessionId));
};
