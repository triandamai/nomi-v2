import { m } from '$lib/paraglide/messages';
import type { UsageMonth } from '$lib/adapters/api';
import { formatTokens } from '$lib/adapters/shell';

// Settings › Plan & usage (App / Settings / Plan & usage C0og49): a month of usage as the pen
// shows it. The allowance (what's used, where this pace ends the month, when it starts again),
// a bar a day, the month's figures and models, and whether the plan fits. Days are the person's
// (the backend counts them in their zone); spend is at list price, in dollars, as the backend
// keeps it.

const utc = (date: string) => new Date(`${date}T00:00:00Z`);
const iso = (at: Date) => at.toISOString().slice(0, 10);
const clamp = (share: number) => Math.min(1, Math.max(0, share));

export const daysIn = (month: string) => new Date(Date.UTC(Number(month.slice(0, 4)), Number(month.slice(5, 7)), 0)).getUTCDate();

/** "October 2026", in the person's language. */
export function monthName(month: string, language: string): string {
	return new Intl.DateTimeFormat(language === 'id' ? 'id-ID' : 'en-US', { month: 'long', year: 'numeric', timeZone: 'UTC' }).format(utc(`${month}-01`));
}

export function shiftMonth(month: string, by: number): string {
	return iso(new Date(Date.UTC(Number(month.slice(0, 4)), Number(month.slice(5, 7)) - 1 + by, 1))).slice(0, 7);
}

/** List-price dollars: "$0.11", "$12.50", "<$0.01" for less than a cent, "$0" for none. */
export function usd(value: number): string {
	if (value <= 0) return '$0';
	if (value < 0.01) return '<$0.01';
	return `$${value.toFixed(2)}`;
}

export type Allowance = { used: number; projected: number; projectedShare: number; resets: string; daysLeft: number; current: boolean };

/** What's used of the allowance, where this pace ends the month, and when it starts again. */
export function allowance(usage: UsageMonth, today: string): Allowance {
	const days = daysIn(usage.month);
	const current = usage.month === today.slice(0, 7);
	const elapsed = current ? Number(today.slice(8, 10)) : days;
	const projected = current ? Math.round((usage.tokens_used / elapsed) * days) : usage.tokens_used;
	const allowed = Math.max(1, usage.plan.monthly_tokens);
	return {
		used: clamp(usage.tokens_used / allowed),
		projected,
		projectedShare: clamp(projected / allowed),
		resets: `${shiftMonth(usage.month, 1)}-01`,
		daysLeft: current ? days - elapsed + 1 : 0,
		current,
	};
}

export type DayBar = { day: number; value: number; share: number; state: 'past' | 'today' | 'future' | 'empty' };

/** A bar for each day of the month against the busiest; today in the accent, the days after it stubs. */
export function dayBars(usage: UsageMonth, today: string, unit: 'tokens' | 'spend'): { bars: DayBar[]; peak: { value: number; date: string } | null } {
	const values = new Map(usage.by_day.map((day) => [day.date, unit === 'tokens' ? day.tokens : day.spend_usd]));
	const most = Math.max(0, ...values.values());
	let peak: { value: number; date: string } | null = null;
	const bars = Array.from({ length: daysIn(usage.month) }, (_, index): DayBar => {
		const date = `${usage.month}-${String(index + 1).padStart(2, '0')}`;
		const value = values.get(date) ?? 0;
		if (value > 0 && (!peak || value > peak.value)) peak = { value, date };
		const state = date === today ? 'today' : date > today ? 'future' : value > 0 ? 'past' : 'empty';
		return { day: index + 1, value, share: most > 0 ? value / most : 0, state };
	});
	return { bars, peak };
}

/** The month's figures: spend at list price, tokens in and out, and the requests. */
export function usageStats(usage: UsageMonth): { label: string; value: string; note: string }[] {
	return [
		{ label: m.plan_stat_spend(), value: usd(usage.spend_usd), note: m.plan_stat_spend_note() },
		{ label: m.plan_stat_in(), value: formatTokens(usage.input_tokens), note: m.plan_stat_in_note() },
		{ label: m.plan_stat_out(), value: formatTokens(usage.output_tokens), note: m.plan_stat_out_note() },
		{ label: m.plan_stat_requests(), value: String(usage.calls), note: m.plan_stat_requests_note({ count: usage.by_model.length }) },
	];
}

/** By model: each one's spend, its share of the month's tokens, and whose key paid for it. */
export function modelSpend(usage: UsageMonth): { label: string; value: string; share: number; note: string }[] {
	const total = Math.max(1, usage.by_model.reduce((sum, model) => sum + model.tokens, 0));
	const most = Math.max(1, ...usage.by_model.map((model) => model.tokens));
	return usage.by_model.map((model) => ({
		label: model.label,
		value: usd(model.spend_usd),
		share: model.tokens / most,
		note: m.plan_model_note({ tokens: formatTokens(model.tokens), share: Math.round((model.tokens / total) * 100), source: model.source === 'own_key' ? m.plan_source_own() : m.plan_source_nomi() }),
	}));
}

/** Whether this pace fits the plan they're on, and if not, the smallest plan that would. */
export function fitNote(projected: number, current: { name: string; monthly_tokens: number }, plans: { name: string; monthly_tokens: number }[]): string {
	const pace = formatTokens(projected);
	if (projected <= current.monthly_tokens) return m.plan_fit_ok({ pace, plan: current.name });
	const bigger = [...plans].sort((a, b) => a.monthly_tokens - b.monthly_tokens).find((plan) => plan.monthly_tokens >= projected);
	const over = m.plan_fit_over({ pace, plan: current.name, allowance: formatTokens(current.monthly_tokens) });
	return bigger ? `${over} ${m.plan_fit_bigger({ plan: bigger.name })}` : over;
}

const cell = (value: string | number) => (/[",\n]/.test(String(value)) ? `"${String(value).replaceAll('"', '""')}"` : String(value));

/** The month as CSV: a row a day, a blank line, then a row a model, the figures as the backend keeps them. */
export function usageCsv(usage: UsageMonth): string {
	return [
		'date,tokens,spend_usd',
		...usage.by_day.map((day) => [day.date, day.tokens, day.spend_usd].map(cell).join(',')),
		'',
		'model,source,tokens,requests,spend_usd',
		...usage.by_model.map((model) => [model.label, model.source, model.tokens, model.calls, model.spend_usd].map(cell).join(',')),
	].join('\n');
}
