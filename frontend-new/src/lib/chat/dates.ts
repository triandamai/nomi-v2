// Times in a chat, where the person is: the day dividers ("Today", "Yesterday", "Sat 17 Oct"),
// a message's clock time, and the live rail's timer. The day divider is the old app's dayLabel
// (frontend/src/lib/dates.ts), counted in the person's zone so the server and the browser agree.

/** A zone Intl knows, else UTC. */
function zone(timeZone: string | undefined): string {
	if (!timeZone) return 'UTC';
	try {
		new Intl.DateTimeFormat('en', { timeZone });
		return timeZone;
	} catch {
		return 'UTC';
	}
}

/** The calendar day `at` falls on in the zone: "2026-10-21". */
export function dayKey(at: string | Date, timeZone?: string): string {
	const parts = Object.fromEntries(
		new Intl.DateTimeFormat('en-GB', { year: 'numeric', month: '2-digit', day: '2-digit', timeZone: zone(timeZone) })
			.formatToParts(new Date(at))
			.map((part) => [part.type, part.value]),
	);
	return `${parts.year}-${parts.month}-${parts.day}`;
}

const DAY_MS = 24 * 60 * 60 * 1000;

/** The intl locale for an app language: day before month, 24-hour clock. */
function intlLocale(language: string): string {
	return language === 'id' ? 'id-ID' : 'en-GB';
}

/**
 * The break between days in a chat: "Today", "Yesterday", else the weekday and date ("Sat 17 Oct"),
 * with the year once it isn't this year. Intl words it in the person's language.
 */
export function dayLabel(at: string | Date, language: string, now: Date, timeZone?: string): string {
	const day = dayKey(at, timeZone);
	const today = dayKey(now, timeZone);
	const daysAgo = Math.round((Date.parse(today) - Date.parse(day)) / DAY_MS);
	if (daysAgo === 0 || daysAgo === 1) {
		const word = new Intl.RelativeTimeFormat(language, { numeric: 'auto' }).format(-daysAgo, 'day');
		return word.charAt(0).toLocaleUpperCase(language) + word.slice(1);
	}
	const year = day.slice(0, 4) === today.slice(0, 4) ? {} : { year: 'numeric' as const };
	const parts = new Intl.DateTimeFormat(intlLocale(language), { weekday: 'short', day: 'numeric', month: 'short', timeZone: zone(timeZone), ...year }).formatToParts(new Date(at));
	const get = (type: string) => parts.find((part) => part.type === type)?.value ?? '';
	return [get('weekday'), get('day'), get('month'), get('year')].filter(Boolean).join(' ');
}

/** The 24-hour time of `at` in the zone: "14:02". */
export function clock(at: string | Date, timeZone?: string): string {
	return new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: zone(timeZone) }).format(new Date(at));
}

/** A running time in the live rail's mono form: "00:04", "01:05". */
export function elapsedLabel(ms: number): string {
	const seconds = Math.max(0, Math.floor(ms / 1000));
	return `${String(Math.floor(seconds / 60)).padStart(2, '0')}:${String(seconds % 60).padStart(2, '0')}`;
}
