import { formatAmount } from '$lib/money';
import type { Format, Scalar } from './types';

// How a view shows a value: as written, or formatted for the person's language. Money comes in
// whole units (45000, 12.5) and goes through the Money page's own formatter, so amounts look the
// same everywhere.

/** An ISO date (2026-10-12) or date-time; anything else isn't one. */
function asDate(value: Scalar): Date | null {
	if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}/.test(value)) return null;
	// A bare date is a calendar day, not midnight UTC (which can fall on the day before).
	const date = /^\d{4}-\d{2}-\d{2}$/.test(value) ? new Date(`${value}T12:00:00`) : new Date(value);
	return Number.isNaN(date.getTime()) ? null : date;
}

function asNumber(value: Scalar): number | null {
	if (typeof value === 'number') return Number.isFinite(value) ? value : null;
	if (typeof value === 'string' && value.trim() !== '' && !Number.isNaN(Number(value))) return Number(value);
	return null;
}

export function formatValue(value: Scalar, format: Format | undefined, locale: string): string {
	if (typeof value === 'boolean') return value ? '✓' : '—';
	switch (format) {
		case 'money': {
			const n = asNumber(value);
			return n === null ? String(value) : formatAmount(Math.round(n * 100));
		}
		case 'number': {
			const n = asNumber(value);
			return n === null ? String(value) : n.toLocaleString(locale);
		}
		case 'percent': {
			const n = asNumber(value);
			if (n === null) return String(value);
			// 0.4 and 40 both mean 40%.
			return (Math.abs(n) <= 1 ? n : n / 100).toLocaleString(locale, { style: 'percent', maximumFractionDigits: 1 });
		}
		case 'date':
		case 'date_time': {
			const date = asDate(value);
			if (!date) return String(value);
			return format === 'date'
				? date.toLocaleDateString(locale, { dateStyle: 'medium' })
				: date.toLocaleString(locale, { dateStyle: 'medium', timeStyle: 'short' });
		}
		default:
			return typeof value === 'number' ? value.toLocaleString(locale) : value;
	}
}

/** A timeline's `at`: a date or date-time in the person's style, or the label as written. */
export function formatWhen(at: string, locale: string): { day: string; time: string | null } {
	const date = asDate(at);
	if (!date) return { day: at, time: null };
	const hasTime = /T\d{2}:\d{2}/.test(at);
	return {
		day: date.toLocaleDateString(locale, { weekday: 'short', day: 'numeric', month: 'short' }),
		time: hasTime ? date.toLocaleTimeString(locale, { hour: '2-digit', minute: '2-digit' }) : null,
	};
}

/** Axis ticks: the largest value rounded up to a round number, in a few steps. */
export function chartScale(max: number, steps = 4): number[] {
	if (!(max > 0)) return [0, 1];
	const raw = max / steps;
	const magnitude = 10 ** Math.floor(Math.log10(raw));
	const step = [1, 2, 2.5, 5, 10].map((f) => f * magnitude).find((s) => s >= raw) ?? raw;
	return Array.from({ length: Math.ceil(max / step) + 1 }, (_, i) => +(i * step).toFixed(10));
}

/** A form's answers as one message, in the field order: "Label: value" lines. */
export function composeForm(title: string | undefined, fields: { key: string; label: string }[], values: Record<string, string>): string {
	const lines = fields.map((f) => [f.label, values[f.key]?.trim() ?? ''] as const).filter(([, v]) => v !== '').map(([label, v]) => `${label}: ${v}`);
	if (lines.length === 0) return '';
	return title ? [title, ...lines].join('\n') : lines.join('\n');
}

/** An axis label: like `formatValue`, but short (12.5K) so ticks fit. */
export function formatTick(value: number, format: Format | undefined, locale: string): string {
	if (format === 'percent') return formatValue(value, format, locale);
	if (format === 'money' || Math.abs(value) >= 10_000) return new Intl.NumberFormat(locale, { notation: 'compact', maximumFractionDigits: 1 }).format(value);
	return value.toLocaleString(locale);
}
