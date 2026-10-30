// A share link's expiry, as the share dialog's chip cycles through it: 7 days, 30, 1, never.
import { m } from '$lib/paraglide/messages';

export type ExpiryDays = 1 | 7 | 30 | null;

const CYCLE: ExpiryDays[] = [7, 30, 1, null];

/** The expiry after `current` when the chip is tapped. */
export function nextExpiry(current: ExpiryDays): ExpiryDays {
	return CYCLE[(CYCLE.indexOf(current) + 1) % CYCLE.length];
}

/** What the chip says: "in 7 days", "in a day", "never". */
export function expiryLabel(days: ExpiryDays): string {
	if (days === null) return m.share_never();
	return days === 1 ? m.share_in_a_day() : m.share_in_days({ count: days });
}

/** The page a link opens: this app's own, read-only (routes/shared/[token]). */
export function shareUrl(origin: string, token: string): string {
	return `${origin}/shared/${token}`;
}

/** A reply's markdown as the plain line the share preview shows: no emphasis, code or link marks. */
export function plainText(markdown: string): string {
	return markdown
		.replace(/\[([^\]]*)\]\([^)]*\)/g, '$1')
		.replace(/(\*\*|__|\*|_|`)/g, '')
		.replace(/^\s{0,3}(#{1,6}|>|[-*+]|\d+\.)\s+/gm, '')
		.replace(/\s+/g, ' ')
		.trim();
}
