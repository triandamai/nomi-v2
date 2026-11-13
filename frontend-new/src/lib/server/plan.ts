import { isRedirect } from '@sveltejs/kit';
import type { OwnConnection, PlansForUser, UsageMonth } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';

// Settings › Plan & usage's calls to the backend, as the person: a month of their usage (this
// one unless asked), the plans and which one they're on, their own provider keys, and when they
// joined. Logic and API use from the old app (frontend/src/routes/(app)/billing).

export type PlanData = { usage: UsageMonth; plans: PlansForUser | null; keys: OwnConnection[] | null; joinedAt: string | null };

async function read<T>(event: ApiEvent, path: string): Promise<T | null> {
	try {
		const response = await apiFetch(event, path);
		return response.ok ? ((await response.json()) as T) : null;
	} catch (error) {
		if (isRedirect(error)) throw error;
		return null;
	}
}

/** Null when the usage can't be read; the plans, keys and join date are left out when they can't. */
export async function loadPlan(event: ApiEvent, month: string | null): Promise<PlanData | null> {
	const query = month && /^\d{4}-\d{2}$/.test(month) ? `?month=${month}` : '';
	const [usage, plans, keys, profile] = await Promise.all([
		read<UsageMonth>(event, `/api/usage${query}`),
		read<PlansForUser>(event, '/api/plans'),
		read<{ connections: OwnConnection[] }>(event, '/api/llm/connections'),
		read<{ joined_at: string }>(event, '/api/profile'),
	]);
	if (!usage) return null;
	return { usage, plans, keys: keys?.connections ?? null, joinedAt: profile?.joined_at ?? null };
}
