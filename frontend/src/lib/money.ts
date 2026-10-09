// Helpers for the Money page (amounts arrive as integer cents).

import type { GradientTone, ShapeName } from './components/m3/shapes';
import { m } from '$lib/paraglide/messages';
import { getLocale } from '$lib/paraglide/runtime';

export function formatAmount(cents: number, options: { compact?: boolean } = {}): string {
	const value = cents / 100;
	if (options.compact && Math.abs(value) >= 10_000) {
		return new Intl.NumberFormat(getLocale(), { notation: 'compact', maximumFractionDigits: 1 }).format(value);
	}
	return new Intl.NumberFormat(getLocale(), { minimumFractionDigits: 2, maximumFractionDigits: 2 }).format(value);
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

/** Every day from `start` to `end` (YYYY-MM-DD, both included): a money month that may run
 * from the 25th to the 24th. `day` counts from 1; `dom` is the day of the month. */
export function fillPeriod(start: string, end: string, days: { date: string; cents: number }[]): { date: string; day: number; dom: number; cents: number }[] {
	const byDate = new Map(days.map((d) => [d.date, d.cents]));
	const out: { date: string; day: number; dom: number; cents: number }[] = [];
	const at = new Date(`${start}T00:00:00Z`);
	const last = new Date(`${end}T00:00:00Z`);
	while (at <= last && out.length < 40) {
		const date = at.toISOString().slice(0, 10);
		out.push({ date, day: out.length + 1, dom: at.getUTCDate(), cents: byDate.get(date) ?? 0 });
		at.setUTCDate(at.getUTCDate() + 1);
	}
	return out;
}

/** "October 2026", or "25 Oct – 24 Nov 2026" when money months don't start on the 1st. */
export function periodLabel(month: string, start: string, end: string, startDay: number): string {
	if (startDay === 1) return monthLabel(month);
	const format = (date: string, withYear: boolean) =>
		new Intl.DateTimeFormat(getLocale(), { day: 'numeric', month: 'short', ...(withYear ? { year: 'numeric' } : {}), timeZone: 'UTC' }).format(new Date(`${date}T00:00:00Z`));
	return `${format(start, false)} – ${format(end, true)}`;
}

/** A receipt line in the add-transaction sheet. */
export interface ItemDraft {
	name: string;
	quantity: string;
	price: string;
}

/** The lines worth sending (named, with a price), and what they add up to in cents. */
export function itemsTotal(items: ItemDraft[]): { items: { name: string; quantity: number; unit_amount: number }[]; cents: number } {
	const filled = items
		.map((i) => ({ name: i.name.trim(), quantity: Number(i.quantity) || 1, unit_amount: Number(i.price) }))
		.filter((i) => i.name && i.unit_amount >= 0 && Number.isFinite(i.unit_amount) && i.quantity > 0);
	return { items: filled, cents: Math.round(filled.reduce((sum, i) => sum + i.quantity * i.unit_amount * 100, 0)) };
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
	if (percent === 0) return { text: m.money_compare_same({ month: previousLabel }), direction: 'same' };
	return change > 0
		? { text: m.money_compare_more({ percent, month: previousLabel }), direction: 'up' }
		: { text: m.money_compare_less({ percent, month: previousLabel }), direction: 'down' };
}

export function shiftMonth(month: string, by: number): string {
	const [year, monthNumber] = month.split('-').map(Number);
	const index = year * 12 + (monthNumber - 1) + by;
	return `${Math.floor(index / 12)}-${String((index % 12) + 1).padStart(2, '0')}`;
}

export function monthLabel(month: string, style: 'long' | 'short' = 'long'): string {
	const [year, monthNumber] = month.split('-').map(Number);
	return new Intl.DateTimeFormat(getLocale(), { month: style, year: 'numeric', timeZone: 'UTC' }).format(new Date(Date.UTC(year, monthNumber - 1, 1)));
}

/** Just the month's name ("September"), in the person's language. */
export function monthName(month: string): string {
	const [year, monthNumber] = month.split('-').map(Number);
	return new Intl.DateTimeFormat(getLocale(), { month: 'long', timeZone: 'UTC' }).format(new Date(Date.UTC(year, monthNumber - 1, 1)));
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

/** A piece of text with an amount's cents (decimal mark and two digits) set apart. */
export interface MoneyPiece {
	text: string;
	cents: boolean;
}

// A digit, then a decimal mark and exactly two digits that end the number: "1,234.56", "Rp 120.000,00".
// "1.000" (thousands), "12,5 rb" (compact) and "$0.0012" have no two-digit cents to raise.
const CENTS = /(?<=\d)([.,]\d{2})(?!\d)/g;

/** Splits text so every amount's cents can be shown raised ("$12" + ".50"). */
export function moneyPieces(text: string): MoneyPiece[] {
	const pieces: MoneyPiece[] = [];
	let at = 0;
	for (const match of text.matchAll(CENTS)) {
		const start = match.index ?? 0;
		if (start > at) pieces.push({ text: text.slice(at, start), cents: false });
		pieces.push({ text: match[0], cents: true });
		at = start + match[0].length;
	}
	if (at < text.length) pieces.push({ text: text.slice(at), cents: false });
	return pieces;
}
