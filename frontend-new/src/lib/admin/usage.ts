import { m } from '$lib/paraglide/messages';
import type { ChartBar } from '$lib/ds/admin/UsageChart.svelte';
import type { AdminUsage } from './api';
import { compact, dayOfMonth, everyDay, full, monthLabel, niceMax, shiftMonth } from './format';

// Usage by month (GET /api/admin/usage?month=): the chart's bars and scale, the month's neighbours.

const dayLabel = (date: string, language: string) =>
	new Intl.DateTimeFormat(language === 'id' ? 'id-ID' : 'en-GB', { weekday: 'short', day: 'numeric', month: 'short', timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`));

/** Chat tokens for every day of the month (the backend lists only days with any use). */
export function usageChart(usage: AdminUsage, language: string) {
	const days = everyDay(usage.month, usage.by_day);
	const max = niceMax(Math.max(0, ...days.map((day) => day.chat_tokens)));
	const bars: ChartBar[] = days.map((day) => {
		const at = dayOfMonth(day.date);
		return {
			key: day.date,
			value: day.chat_tokens,
			...((at === 1 || at % 7 === 0) && { tick: String(at) }),
			tooltip: [dayLabel(day.date, language), m.admin_usage_day_chat({ count: full(day.chat_tokens, language) }), m.admin_usage_day_embedding({ count: full(day.embedding_tokens, language) })],
		};
	});
	return {
		bars,
		max,
		axis: [compact(max, language), compact(max / 2, language), '0'] as [string, string, string],
		/** The same figures as a table: the days with any use. */
		days: usage.by_day.map((day) => ({ date: day.date, label: dayLabel(day.date, language), chat: full(day.chat_tokens, language), embedding: full(day.embedding_tokens, language) })),
	};
}

/** The months either side: back without end, forward up to this month. */
export function monthLinks(usage: AdminUsage, language = 'en') {
	return {
		previous: `?month=${shiftMonth(usage.month, -1)}`,
		next: usage.month < usage.current_month ? `?month=${shiftMonth(usage.month, 1)}` : null,
		label: monthLabel(usage.month, language),
	};
}

/** Nobody used anything this month. */
export const quiet = (usage: AdminUsage) => usage.chat.calls === 0 && usage.embedding.calls === 0;
