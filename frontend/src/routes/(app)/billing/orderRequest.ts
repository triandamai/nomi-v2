/** What a quote or a checkout is asked for: the plan, how many months, and a code if one was entered. */
export interface OrderRequest {
	plan_id: string;
	months: number;
	code?: string;
}

/** The body of a quote or checkout request, or null when it isn't one. */
export function readOrderRequest(body: unknown): OrderRequest | null {
	if (typeof body !== 'object' || body === null) return null;
	const { plan_id, months, code } = body as Record<string, unknown>;
	if (typeof plan_id !== 'string' || !plan_id) return null;
	if (typeof months !== 'number' || !Number.isInteger(months) || months < 1) return null;
	const trimmed = typeof code === 'string' ? code.trim() : '';
	return { plan_id, months, ...(trimmed ? { code: trimmed } : {}) };
}
