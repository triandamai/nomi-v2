import { isRedirect } from '@sveltejs/kit';
import type { OwnConnection, PaymentResult, PlansForUser, UsageMonth } from '$lib/adapters/api';
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

async function post(event: ApiEvent, path: string): Promise<Response | null> {
	try {
		return await apiFetch(event, path, { method: 'POST' });
	} catch (error) {
		if (isRedirect(error)) throw error;
		return null;
	}
}

/** Asks to be told when a plan without a price opens; false when that couldn't be saved. */
export async function joinWaitlist(event: ApiEvent, planId: string): Promise<boolean> {
	return Boolean((await post(event, `/api/plans/${encodeURIComponent(planId)}/waitlist`))?.ok);
}

/**
 * The payment gateway's page to pay a month of a plan on, or the backend's reason it can't start
 * (`already_on_plan`, `not_for_sale`, `payments_not_configured`, `gateway_error`). The page is only
 * ever an https address: the person is sent straight to it.
 */
export async function startCheckout(event: ApiEvent, planId: string): Promise<{ url: string } | { error: string; status: number }> {
	const response = await post(event, `/api/plans/${encodeURIComponent(planId)}/checkout`);
	if (!response) return { error: 'gateway_error', status: 503 };
	const body = (await response.json().catch(() => ({}))) as { url?: string; error?: string };
	if (!response.ok) return { error: body.error ?? 'gateway_error', status: response.status };
	try {
		if (body.url && new URL(body.url).protocol === 'https:') return { url: body.url };
	} catch {
		// Not an address: refused below.
	}
	return { error: 'gateway_error', status: 502 };
}

/** A payment of the person's (the gateway sends them back with its id); null for any other. */
export async function loadPayment(event: ApiEvent, id: string): Promise<PaymentResult | null> {
	if (!/^[0-9a-zA-Z-]+$/.test(id)) return null;
	return read<PaymentResult>(event, `/api/billing/payments/${id}`);
}
