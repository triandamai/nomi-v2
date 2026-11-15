// Settings › Plan & usage: joining a plan (App / Settings / Plan & usage C0og49 → Compare plans).
// A plan with a price is bought on the payment gateway's page; one without a price has a waitlist
// (the pen's "Join the waitlist"); coming back from paying, the page says what the payment came to.

export type PlanChoice = 'theirs' | 'buy' | 'closed' | 'waitlist' | 'waiting' | 'none';

/** What a plan offers the person: theirs, buying it, its waitlist, or nothing (a smaller or free plan). */
export function planChoice(
	plan: { id: string; is_default: boolean; monthly_tokens: number; price_idr: number | null },
	ctx: { currentId: string | null; currentTokens: number | null; waitlist: string[]; checkoutAvailable: boolean },
): PlanChoice {
	if (plan.id === ctx.currentId) return 'theirs';
	if (plan.price_idr === null) {
		if (plan.is_default) return 'none';
		return ctx.waitlist.includes(plan.id) ? 'waiting' : 'waitlist';
	}
	if (ctx.currentTokens !== null && plan.monthly_tokens <= ctx.currentTokens) return 'none';
	return ctx.checkoutAvailable ? 'buy' : 'closed';
}

/**
 * How much more room a plan gives than theirs ("10×", the pen's Pro card, where a plan without a
 * price shows its room in the price's place); null when it isn't more or theirs isn't known.
 */
export function roomOf(tokens: number, theirs: number | null): string | null {
	if (!theirs || tokens <= theirs) return null;
	return `${new Intl.NumberFormat('en-US', { maximumFractionDigits: 1 }).format(tokens / theirs)}×`;
}

/** How a payment is told on the page, and whether its payment page can be opened again (only while it waits). */
export function paymentOutcome(payment: { status: string; invoice_url: string | null }): { kind: 'success' | 'info' | 'warning' | 'error'; open: boolean } {
	const safe = (() => {
		try {
			return payment.invoice_url !== null && new URL(payment.invoice_url).protocol === 'https:';
		} catch {
			return false;
		}
	})();
	if (payment.status === 'paid') return { kind: 'success', open: false };
	if (payment.status === 'pending') return { kind: 'info', open: safe };
	if (payment.status === 'failed') return { kind: 'error', open: false };
	return { kind: 'warning', open: false };
}
