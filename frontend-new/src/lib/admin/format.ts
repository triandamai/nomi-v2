// How the admin console writes numbers, money, months and times: in the person's language, with
// the day before the month and a 24-hour clock in both (as the app shell does).

const intl = (language: string) => (language === 'id' ? 'id-ID' : 'en-GB');

/** "1.2K", "2.5M" (en); "1,2 rb" (id). */
export function compact(value: number, language: string): string {
	return new Intl.NumberFormat(language === 'id' ? 'id-ID' : 'en-US', { notation: 'compact', maximumFractionDigits: 1 }).format(value);
}

/** "1,234,567" (en); "1.234.567" (id). */
export function full(value: number, language: string): string {
	return new Intl.NumberFormat(language === 'id' ? 'id-ID' : 'en-US', { maximumFractionDigits: 0 }).format(value);
}

/** US dollars, as the backend counts spend: to the cent, or 4 places under a cent. */
export function usd(value: number, language: string): string {
	const tiny = value > 0 && value < 0.01;
	return new Intl.NumberFormat(language === 'id' ? 'id-ID' : 'en-US', {
		style: 'currency',
		currency: 'USD',
		minimumFractionDigits: tiny ? 4 : 2,
		maximumFractionDigits: tiny ? 4 : 2,
	}).format(value);
}

/** `YYYY-MM` moved by `by` months. */
export function shiftMonth(month: string, by: number): string {
	const [year, index] = month.split('-').map(Number);
	const at = new Date(Date.UTC(year, index - 1 + by, 1));
	return `${at.getUTCFullYear()}-${String(at.getUTCMonth() + 1).padStart(2, '0')}`;
}

/** "November 2026" (UTC, as the backend's months are). */
export function monthLabel(month: string, language: string): string {
	const [year, index] = month.split('-').map(Number);
	return new Intl.DateTimeFormat(intl(language), { month: 'long', year: 'numeric', timeZone: 'UTC' }).format(new Date(Date.UTC(year, index - 1, 1)));
}

export type DayUse = { date: string; chat_tokens: number; embedding_tokens: number };

/** Every day of `month`, with what was used on it (the backend lists only days with any use). */
export function everyDay(month: string, days: DayUse[]): DayUse[] {
	const [year, index] = month.split('-').map(Number);
	const count = new Date(Date.UTC(year, index, 0)).getUTCDate();
	const used = new Map(days.map((day) => [day.date, day]));
	return Array.from({ length: count }, (_, at) => {
		const date = `${month}-${String(at + 1).padStart(2, '0')}`;
		return used.get(date) ?? { date, chat_tokens: 0, embedding_tokens: 0 };
	});
}

export const dayOfMonth = (date: string): number => Number(date.slice(8, 10));

/** The chart's top: the largest value rounded up to 1, 2 or 5 times a power of ten. */
export function niceMax(value: number): number {
	if (value <= 1) return 1;
	const power = 10 ** Math.floor(Math.log10(value));
	const step = [1, 2, 5, 10].find((factor) => factor * power >= value) ?? 10;
	return step * power;
}

/** A time zone Intl knows, else UTC. */
function zone(timeZone: string | undefined): string {
	if (!timeZone) return 'UTC';
	try {
		new Intl.DateTimeFormat('en', { timeZone });
		return timeZone;
	} catch {
		return 'UTC';
	}
}

/** "10 Nov 2026, 08:15" where the person is; "—" for none. */
export function dateTime(iso: string | null | undefined, language: string, timeZone?: string): string {
	if (!iso) return '—';
	const at = new Date(iso);
	if (Number.isNaN(at.getTime())) return '—';
	return new Intl.DateTimeFormat(intl(language), { day: 'numeric', month: 'short', year: 'numeric', hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: zone(timeZone) }).format(at);
}

/** "10 Nov 2026" where the person is; "—" for none. */
export function dateOnly(iso: string | null | undefined, language: string, timeZone?: string): string {
	if (!iso) return '—';
	const at = new Date(iso);
	if (Number.isNaN(at.getTime())) return '—';
	return new Intl.DateTimeFormat(intl(language), { day: 'numeric', month: 'short', year: 'numeric', timeZone: zone(timeZone) }).format(at);
}

/** The wall-clock date and time of an instant in a zone. */
function wallClock(at: Date, timeZone: string): { date: string; time: string } {
	const parts = Object.fromEntries(
		new Intl.DateTimeFormat('en-GB', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone })
			.formatToParts(at)
			.map((part) => [part.type, part.value]),
	);
	return { date: `${parts.year}-${parts.month}-${parts.day}`, time: `${parts.hour}:${parts.minute}` };
}

/** A date (`YYYY-MM-DD`) and time (`HH:MM`, midnight when empty) typed in `timeZone`, as an instant. */
export function isoFromLocal(date: string, time: string, timeZone?: string): string | null {
	const day = date.match(/^(\d{4})-(\d{2})-(\d{2})$/);
	if (!day) return null;
	const clock = (time || '00:00').match(/^(\d{2}):(\d{2})$/);
	if (!clock) return null;
	const wanted = Date.UTC(Number(day[1]), Number(day[2]) - 1, Number(day[3]), Number(clock[1]), Number(clock[2]));
	const tz = zone(timeZone);
	// The zone's offset at that moment: read the guess back on the zone's clock and move by the gap.
	let guess = wanted;
	for (let round = 0; round < 2; round++) {
		const seen = wallClock(new Date(guess), tz);
		const [y, mo, d] = seen.date.split('-').map(Number);
		const [h, mi] = seen.time.split(':').map(Number);
		guess += wanted - Date.UTC(y, mo - 1, d, h, mi);
	}
	return new Date(guess).toISOString();
}

/** An instant as the date and time fields show it in `timeZone`; empty for none. */
export function localParts(iso: string | null | undefined, timeZone?: string): { date: string; time: string } {
	if (!iso) return { date: '', time: '' };
	const at = new Date(iso);
	return Number.isNaN(at.getTime()) ? { date: '', time: '' } : wallClock(at, zone(timeZone));
}

/** A date and time typed in UTC, as an instant: null unless they read back the same (no 25:00, no 31 February). */
export function utcInstant(date: string, time: string): string | null {
	const iso = isoFromLocal(date, time, 'UTC');
	if (!iso) return null;
	const back = localParts(iso, 'UTC');
	return back.date === date && back.time === (time || '00:00') ? iso : null;
}

/** The last second of a day in UTC: when an allowance or a promotion ends. */
export function endOfDayUtc(date: string): string | null {
	const lastMinute = utcInstant(date, '23:59');
	return lastMinute ? new Date(Date.parse(lastMinute) + 59_000).toISOString() : null;
}

/** A plan's id from its name: lower case, a dash for anything else, none at the ends. */
export function slugify(text: string): string {
	return text
		.toLowerCase()
		.replace(/[^a-z0-9]+/g, '-')
		.replace(/^-+|-+$/g, '');
}
