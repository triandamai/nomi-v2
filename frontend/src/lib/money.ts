// Helpers for the Money page (amounts arrive as integer cents).

import type { GradientTone, ShapeName } from './components/m3/shapes';

export function formatAmount(cents: number, options: { compact?: boolean } = {}): string {
	const value = cents / 100;
	if (options.compact && Math.abs(value) >= 10_000) {
		return new Intl.NumberFormat(undefined, { notation: 'compact', maximumFractionDigits: 1 }).format(value);
	}
	return new Intl.NumberFormat(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(value);
}

/** Every day of `month` (YYYY-MM), with zero for days without spending. */
export function fillDays(month: string, days: { date: string; cents: number }[]): { date: string; day: number; cents: number }[] {
	const [year, monthIndex] = month.split('-').map(Number);
	const count = new Date(Date.UTC(year, monthIndex, 0)).getUTCDate();
	const byDate = new Map(days.map((d) => [d.date, d.cents]));
	return Array.from({ length: count }, (_, i) => {
		const date = `${month}-${String(i + 1).padStart(2, '0')}`;
		return { date, day: i + 1, cents: byDate.get(date) ?? 0 };
	});
}

/** A clean axis maximum at or above `max` (1, 2, 2.5 or 5 × a power of ten) and its ticks. */
export function niceScale(max: number, tickCount = 4): { max: number; ticks: number[] } {
	if (max <= 0) return { max: 1, ticks: [0, 1] };
	const rough = max / tickCount;
	const power = 10 ** Math.floor(Math.log10(rough));
	const step = [1, 2, 2.5, 5, 10].map((m) => m * power).find((s) => s >= rough) ?? 10 * power;
	const top = Math.ceil(max / step) * step;
	const ticks: number[] = [];
	for (let t = 0; t <= top + step / 2; t += step) ticks.push(Math.round(t * 100) / 100);
	return { max: top, ticks };
}

/** "18% less than September" style comparison; null when there's nothing to compare. */
export function compareMonths(current: number, previous: number, previousLabel: string): { text: string; direction: 'up' | 'down' | 'same' } | null {
	if (previous <= 0) return null;
	const change = (current - previous) / previous;
	const percent = Math.round(Math.abs(change) * 100);
	if (percent === 0) return { text: `About the same as ${previousLabel}`, direction: 'same' };
	return change > 0
		? { text: `${percent}% more than ${previousLabel}`, direction: 'up' }
		: { text: `${percent}% less than ${previousLabel}`, direction: 'down' };
}

export function shiftMonth(month: string, by: number): string {
	const [year, m] = month.split('-').map(Number);
	const index = year * 12 + (m - 1) + by;
	return `${Math.floor(index / 12)}-${String((index % 12) + 1).padStart(2, '0')}`;
}

export function monthLabel(month: string, style: 'long' | 'short' = 'long'): string {
	const [year, m] = month.split('-').map(Number);
	return new Intl.DateTimeFormat(undefined, { month: style, year: 'numeric', timeZone: 'UTC' }).format(new Date(Date.UTC(year, m - 1, 1)));
}

const CATEGORY_LOOKS: Record<string, { shape: ShapeName; tone: GradientTone }> = {
	food: { shape: 'cookie9', tone: 'citrus' },
	groceries: { shape: 'cookie9', tone: 'citrus' },
	transport: { shape: 'pill', tone: 'sky' },
	travel: { shape: 'clover4', tone: 'tide' },
	subscriptions: { shape: 'burst16', tone: 'dusk' },
	shopping: { shape: 'soft-square', tone: 'bloom' },
	bills: { shape: 'pentagon', tone: 'slate' },
	health: { shape: 'puffy7', tone: 'glow' },
	entertainment: { shape: 'sunny8', tone: 'ember' },
};
const FALLBACK_LOOKS: { shape: ShapeName; tone: GradientTone }[] = [
	{ shape: 'flower5', tone: 'bloom' },
	{ shape: 'wave10', tone: 'tide' },
	{ shape: 'soft-triangle', tone: 'ember' },
	{ shape: 'sunny12', tone: 'sky' },
];

/** The shape + gradient a spending category wears, stable for unknown categories too. */
export function categoryLook(category: string): { shape: ShapeName; tone: GradientTone } {
	const key = category.toLowerCase();
	if (CATEGORY_LOOKS[key]) return CATEGORY_LOOKS[key];
	let hash = 0;
	for (const ch of key) hash = (hash * 31 + ch.charCodeAt(0)) >>> 0;
	return FALLBACK_LOOKS[hash % FALLBACK_LOOKS.length];
}

/** How a budget stands this month. */
export function budgetState(spent: number, limit: number): { ratio: number; over: boolean; nearly: boolean } {
	const ratio = limit > 0 ? spent / limit : 0;
	return { ratio, over: ratio > 1, nearly: ratio >= 0.8 && ratio <= 1 };
}
