/** The profile heatmap's "Most active" note: the busiest weekday, and the hours when known. */
import type { ActiveHours } from './types';

const DAYS_IN_WEEK = 7;

/** The weekday (0 = Sunday) with the most tokens, or null with none. Dates are "YYYY-MM-DD". */
export function busiestWeekday(activity: { date: string; tokens: number }[]): number | null {
	const totals = new Array<number>(DAYS_IN_WEEK).fill(0);
	for (const day of activity) {
		const [year, month, date] = day.date.split('-').map(Number);
		if (!year || !month || !date) continue;
		totals[new Date(year, month - 1, date).getDay()] += day.tokens;
	}
	const best = Math.max(...totals);
	return best > 0 ? totals.indexOf(best) : null;
}

/** "Tue · 21:00–23:00", "Tue", or null when there's nothing to say yet. */
export function mostActiveLabel(activity: { date: string; tokens: number }[], hours: ActiveHours | null | undefined, locale: string): string | null {
	const weekday = busiestWeekday(activity);
	if (weekday === null) return hours ? `${hours.from}–${hours.to}` : null;
	// 2026-11-01 is a Sunday: add the weekday to name it.
	const name = new Intl.DateTimeFormat(locale, { weekday: 'short' }).format(new Date(2026, 10, 1 + weekday));
	return hours ? `${name} · ${hours.from}–${hours.to}` : name;
}
