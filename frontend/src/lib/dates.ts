// Plain calendar dates ("2026-10-10") and times ("14:30") for the date and time pickers: no
// time zones, just the day and the clock the person picked. Dates compare as strings.

const pad = (n: number) => String(n).padStart(2, '0');

export function toISODate(date: Date): string {
	return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}`;
}

/** The day a "YYYY-MM-DD" names, at local midnight; null when it isn't a real day. */
export function parseISODate(value: string | null | undefined): Date | null {
	const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value ?? '');
	if (!match) return null;
	const [year, month, day] = [Number(match[1]), Number(match[2]) - 1, Number(match[3])];
	const date = new Date(year, month, day);
	return date.getFullYear() === year && date.getMonth() === month && date.getDate() === day ? date : null;
}

export function todayISO(now = new Date()): string {
	return toISODate(now);
}

export function isoFrom(year: number, month: number, day: number): string {
	return toISODate(new Date(year, month, Math.min(day, daysInMonth(year, month))));
}

/** `month` is 0-based, like Date's. */
export function daysInMonth(year: number, month: number): number {
	return new Date(year, month + 1, 0).getDate();
}

export function addMonths(year: number, month: number, delta: number): { year: number; month: number } {
	const date = new Date(year, month + delta, 1);
	return { year: date.getFullYear(), month: date.getMonth() };
}

export interface GridDay {
	iso: string;
	day: number;
	inMonth: boolean;
}

/** Six weeks covering the month, starting on `weekStart` (0 = Sunday, 1 = Monday). */
export function monthGrid(year: number, month: number, weekStart: number): GridDay[] {
	const first = new Date(year, month, 1);
	const lead = (first.getDay() - weekStart + 7) % 7;
	return Array.from({ length: 42 }, (_, i) => {
		const date = new Date(year, month, 1 - lead + i);
		return { iso: toISODate(date), day: date.getDate(), inMonth: date.getMonth() === month };
	});
}

/** The day the week starts on where `locale` is spoken. */
export function weekStartFor(locale: string): number {
	try {
		const info = (new Intl.Locale(locale) as Intl.Locale & { getWeekInfo?: () => { firstDay: number }; weekInfo?: { firstDay: number } });
		const firstDay = info.getWeekInfo?.().firstDay ?? info.weekInfo?.firstDay;
		if (firstDay) return firstDay % 7;
	} catch {
		// Older browsers: guess below.
	}
	return locale.startsWith('en') ? 0 : 1;
}

export function weekdayLabels(locale: string, weekStart: number, style: 'narrow' | 'short' | 'long' = 'narrow'): string[] {
	const format = new Intl.DateTimeFormat(locale, { weekday: style, timeZone: 'UTC' });
	// 2023-01-01 was a Sunday.
	return Array.from({ length: 7 }, (_, i) => format.format(new Date(Date.UTC(2023, 0, 1 + ((weekStart + i) % 7)))));
}

export function monthLabels(locale: string, style: 'long' | 'short' = 'long'): string[] {
	const format = new Intl.DateTimeFormat(locale, { month: style, timeZone: 'UTC' });
	return Array.from({ length: 12 }, (_, i) => format.format(new Date(Date.UTC(2023, i, 1))));
}

export function formatDate(iso: string, locale: string, options: Intl.DateTimeFormatOptions = { dateStyle: 'medium' }): string {
	const date = parseISODate(iso);
	return date ? new Intl.DateTimeFormat(locale, options).format(date) : '';
}

/** Which order day, month and year are written in where `locale` is spoken. */
export function dateFieldOrder(locale: string): ('day' | 'month' | 'year')[] {
	return new Intl.DateTimeFormat(locale, { day: 'numeric', month: 'long', year: 'numeric' })
		.formatToParts(new Date(2023, 0, 15))
		.map((p) => p.type)
		.filter((t): t is 'day' | 'month' | 'year' => t === 'day' || t === 'month' || t === 'year');
}

export function clampDate(iso: string, min?: string, max?: string): string {
	if (min && iso < min) return min;
	if (max && iso > max) return max;
	return iso;
}

/** A range from two picked days, in order. */
export function orderRange(a: string, b: string): [string, string] {
	return a <= b ? [a, b] : [b, a];
}

export function parseTime(value: string | null | undefined): { hour: number; minute: number } | null {
	const match = /^(\d{1,2}):(\d{2})/.exec(value ?? '');
	if (!match) return null;
	const [hour, minute] = [Number(match[1]), Number(match[2])];
	return hour < 24 && minute < 60 ? { hour, minute } : null;
}

export function formatTime(hour: number, minute: number): string {
	return `${pad(hour)}:${pad(minute)}`;
}

/** Whether clocks read 2:30 PM (true) or 14:30 where `locale` is spoken. */
export function uses12Hour(locale: string): boolean {
	return new Intl.DateTimeFormat(locale, { hour: 'numeric' }).resolvedOptions().hour12 === true;
}

export function displayTime(value: string, locale: string): string {
	const time = parseTime(value);
	if (!time) return '';
	return new Intl.DateTimeFormat(locale, { hour: 'numeric', minute: '2-digit' }).format(new Date(2023, 0, 1, time.hour, time.minute));
}

/** The hour on a 12-hour clock (12, 1 … 11) and whether it's after noon. */
export function to12Hour(hour: number): { hour: number; pm: boolean } {
	return { hour: hour % 12 === 0 ? 12 : hour % 12, pm: hour >= 12 };
}

export function from12Hour(hour: number, pm: boolean): number {
	return (hour % 12) + (pm ? 12 : 0);
}
