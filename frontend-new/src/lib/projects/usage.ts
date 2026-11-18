import type { ProjectUsage } from '$lib/adapters/api';
import { formatTokens } from '$lib/adapters/shell';
import { full, monthLabel } from '$lib/admin/format';
import { usd } from '$lib/settings/plan';

// A project's Tokens & cost sheet: a month of what the crew used in its chat, the chats moved
// into it and its issues' chats. Spend is in rupiah; the month's total also gives the list-price
// dollars the backend keeps. Calls from before calls carried their chat count toward no project,
// so the sheet says since when it counts instead of showing a zero that looks like nothing
// happened.

/** "Rp 81.600": whole rupiah, grouped the Indonesian way in either language. */
export function rupiah(amount: number): string {
	return `Rp ${new Intl.NumberFormat('id-ID', { maximumFractionDigits: 0 }).format(Math.round(amount))}`;
}

export type UsageModelRow = { key: string; label: string; ownKey: boolean; tokens: string; spend: string };
export type UsagePersonRow = { id: string; name: string | null; tokens: string; spend: string };

export type UsageView = {
	month: string;
	/** The month shown is this month: there's no next one to step to. */
	current: boolean;
	spend: string;
	spendUsd: string;
	tokens: string;
	calls: string;
	/** Tokens sent with someone's own key, when any were. */
	ownKey: string | null;
	empty: boolean;
	models: UsageModelRow[];
	/** For the project's admins; null for everyone else. */
	people: UsagePersonRow[] | null;
	/** "17 Nov 2026", or null when nothing has been counted yet. */
	since: string | null;
};

export function usageView(usage: ProjectUsage, language: string): UsageView {
	return {
		month: monthLabel(usage.month, language),
		current: usage.month === usage.current_month,
		spend: rupiah(usage.spend_usd * usage.usd_to_idr),
		spendUsd: usd(usage.spend_usd),
		tokens: formatTokens(usage.tokens),
		calls: full(usage.calls, language),
		ownKey: usage.own_key_tokens > 0 ? formatTokens(usage.own_key_tokens) : null,
		empty: usage.calls === 0,
		models: usage.by_model.map((row) => ({
			key: `${row.admin_model_id}-${row.provider}-${row.model_id}-${row.source}`,
			label: row.label,
			ownKey: row.source === 'own_key',
			tokens: formatTokens(row.tokens),
			spend: rupiah(row.spend_usd * usage.usd_to_idr),
		})),
		people: usage.shows_people ? usage.by_person.map((row) => ({ id: row.user_id, name: row.name, tokens: formatTokens(row.tokens), spend: rupiah(row.spend_usd * usage.usd_to_idr) })) : null,
		since: usage.counted_since
			? new Intl.DateTimeFormat('en-GB', { day: 'numeric', month: 'short', year: 'numeric', timeZone: usage.timezone }).format(new Date(usage.counted_since))
			: null,
	};
}
