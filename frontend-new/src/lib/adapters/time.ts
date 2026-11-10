import { m } from '$lib/paraglide/messages';
import { getLocale } from '$lib/paraglide/runtime';

// Times as the app shell shows them: how long ago in the sidebar ("now", "2h", "3d"), the part of
// the day for the greeting, and the top bar's date and time, all where the person is.

const MINUTE = 60_000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;
const WEEK = 7 * DAY;
/** Past this, a recent shows its date instead of how long ago. */
const MONTH = 30 * DAY;

/** The formatting locale for an app language: day before month, 24-hour clock, in both. */
function intlLocale(language: string): string {
	return language === 'id' ? 'id-ID' : 'en-GB';
}

/** A time zone Intl knows, else UTC (an account's zone may be one this runtime lacks). */
function zone(timeZone: string | undefined): string {
	if (!timeZone) return 'UTC';
	try {
		new Intl.DateTimeFormat('en', { timeZone });
		return timeZone;
	} catch {
		return 'UTC';
	}
}

/** How long ago `iso` was, in the sidebar's short mono form: now, 5m, 2h, 3d, 2w, then "2 Aug". */
export function ago(iso: string, now: Date, timeZone?: string): string {
	const elapsed = now.getTime() - Date.parse(iso);
	if (elapsed < MINUTE) return m.time_now();
	if (elapsed < HOUR) return m.time_minutes({ count: Math.floor(elapsed / MINUTE) });
	if (elapsed < DAY) return m.time_hours({ count: Math.floor(elapsed / HOUR) });
	if (elapsed < WEEK) return m.time_days({ count: Math.floor(elapsed / DAY) });
	if (elapsed < MONTH) return m.time_weeks({ count: Math.floor(elapsed / WEEK) });
	return new Intl.DateTimeFormat(intlLocale(getLocale()), { day: 'numeric', month: 'short', timeZone: zone(timeZone) }).format(new Date(iso));
}

export type DayPart = 'morning' | 'midday' | 'afternoon' | 'evening';

/**
 * The part of the day where the person is: morning from 4, midday from 12, afternoon from 15,
 * evening from 18. Indonesian names each (pagi, siang, sore, malam); English says "Afternoon" for
 * both middle ones.
 */
export function dayPart(now: Date, timeZone?: string): DayPart {
	const hour = Number(new Intl.DateTimeFormat('en-GB', { hour: '2-digit', hourCycle: 'h23', timeZone: zone(timeZone) }).format(now));
	if (hour >= 4 && hour < 12) return 'morning';
	if (hour >= 12 && hour < 15) return 'midday';
	if (hour >= 15 && hour < 18) return 'afternoon';
	return 'evening';
}

/** Home's greeting: "Morning, Mario." */
export function greeting(name: string, part: DayPart): string {
	const say = { morning: m.greeting_morning, midday: m.greeting_midday, afternoon: m.greeting_afternoon, evening: m.greeting_evening }[part];
	return say({ name });
}

/** The top bar's date and time: "Wed, 21 Oct 2026 · 08:14" (Indonesian: "Rab, 21 Okt 2026 · 15:14"). */
export function stamp(now: Date, language: string, timeZone?: string): string {
	const parts = Object.fromEntries(
		new Intl.DateTimeFormat(intlLocale(language), {
			weekday: 'short',
			day: 'numeric',
			month: 'short',
			year: 'numeric',
			hour: '2-digit',
			minute: '2-digit',
			hourCycle: 'h23',
			timeZone: zone(timeZone),
		})
			.formatToParts(now)
			.map((part) => [part.type, part.value]),
	);
	return `${parts.weekday}, ${parts.day} ${parts.month} ${parts.year} · ${parts.hour}:${parts.minute}`;
}
