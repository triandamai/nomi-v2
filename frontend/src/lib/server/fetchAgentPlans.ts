import { error } from '@sveltejs/kit';
import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { renderMarkdown } from '$lib/server/markdown';
import type { AgentPlansResponse } from '$lib/types';

/** Fetches every plan version for one agent_session and server-renders each version's markdown
 * body — mirrors fetchRenderedMessage.ts's pattern (raw content from the backend, HTML rendered
 * here before it ever reaches the client). A version whose content is `null` (an S3 read failure
 * on the backend) keeps `content_html: null` — the side sheet shows an inline "could not load
 * this version" state for that one entry rather than failing the whole list. */
export async function fetchAgentPlans(
	fetch: typeof globalThis.fetch,
	cookies: Cookies,
	sessionId: string,
	agentSessionId: string,
): Promise<{ id: string; title: string; version: number; content_html: string | null; created_at: string }[]> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${sessionId}/agent-plans/${agentSessionId}`);
	if (!response.ok) {
		throw error(response.status, 'Could not load this plan.');
	}
	const { plans } = (await response.json()) as AgentPlansResponse;
	return Promise.all(
		plans.map(async (plan) => ({
			id: plan.id,
			title: plan.title,
			version: plan.version,
			content_html: plan.content !== null ? await renderMarkdown(plan.content) : null,
			created_at: plan.created_at,
		})),
	);
}
