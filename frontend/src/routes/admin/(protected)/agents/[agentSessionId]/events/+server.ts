import { json } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ params, url, cookies, fetch }) => {
	const sessionId = url.searchParams.get('sessionId');
	const query = sessionId ? `?session_id=${sessionId}` : '';
	const response = await apiFetch(fetch, cookies, `/api/admin/agent-events${query}`);
	if (!response.ok) {
		return json([], { status: response.status });
	}
	const events = await response.json();
	// The history endpoint doesn't filter by agent_session_id (only session_id) — narrow to this
	// one agent here, since a session can in principle have had more than one agent over time.
	const filtered = Array.isArray(events) ? events.filter((e) => e.agent_session_id === params.agentSessionId) : [];
	return json(filtered);
};
