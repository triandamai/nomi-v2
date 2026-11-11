// The Inbox's quick reminder (App / Inbox BFoiZ → Quick Add): one line, as a person would say it
// ("pay PLN electricity every 25th at 10am", "besok jam 4 sore telepon ibu"), read into what the
// backend's POST /api/reminders takes: a title, a local date and time (the backend reads it in the
// person's zone) and daily, weekly or monthly. English and Indonesian, whichever the person types.
// It reads only what it's sure of: a day with no time stays without one, for the person to add.

export type Repeat = 'daily' | 'weekly' | 'monthly';
/** `date` is "2026-10-25" and `time` "10:00", where the person is. */
export type ParsedReminder = { title: string; date?: string; time?: string; repeat?: Repeat };
export type LocalNow = { date: string; time: string };

const WEEKDAYS: Record<string, number> = {
	sun: 0, sunday: 0, minggu: 0, ahad: 0,
	mon: 1, monday: 1, senin: 1,
	tue: 2, tues: 2, tuesday: 2, selasa: 2,
	wed: 3, wednesday: 3, rabu: 3,
	thu: 4, thur: 4, thurs: 4, thursday: 4, kamis: 4,
	fri: 5, friday: 5, jumat: 5, "jum'at": 5,
	sat: 6, saturday: 6, sabtu: 6,
};
const FULL_DAY = "sunday|monday|tuesday|wednesday|thursday|friday|saturday|senin|selasa|rabu|kamis|jumat|jum'at|sabtu";
const ANY_DAY = `${FULL_DAY}|minggu|ahad|sun|mon|tues|tue|wed|thurs|thur|thu|fri|sat`;
const ORDINAL = '(\\d{1,2})(?:st|nd|rd|th)';

type Found = { repeat?: Repeat; weekday?: number; monthDay?: number; dayOffset?: number; minutesFromNow?: number; time?: string };

/** "10:00" from an hour, minutes and what came after (am/pm, pagi/siang/sore/malam); nothing if it isn't a time. */
function clockTime(hourText: string, minuteText: string | undefined, part: string | undefined): string | undefined {
	let hour = Number(hourText);
	const minute = Number(minuteText ?? 0);
	const said = part?.toLowerCase().replaceAll('.', '');
	if (said === 'pm' || said === 'sore' || (said === 'siang' && hour < 11) || (said === 'malam' && hour >= 6 && hour < 12)) hour = hour === 12 ? 12 : hour + 12;
	if ((said === 'am' || said === 'malam') && hour === 12) hour = 0;
	if (hour > 23 || minute > 59) return undefined;
	return `${String(hour).padStart(2, '0')}:${String(minute).padStart(2, '0')}`;
}

type Rule = { pattern: RegExp; read: (match: RegExpExecArray, found: Found) => boolean };

const PART = '(am|pm|a\\.m\\.|p\\.m\\.|pagi|siang|sore|malam)';
const RULES: Rule[] = [
	// "in 10 minutes", "dalam 2 jam (lagi)", "2 jam lagi"
	{
		pattern: /\b(?:in|dalam)\s+(\d{1,3})\s*(minutes?|mins?|hours?|hrs?|menit|jam)\b(?:\s+lagi\b)?|\b(\d{1,3})\s*(menit|jam)\s+lagi\b/i,
		read: (m, found) => {
			const [count, unit] = m[1] ? [m[1], m[2]] : [m[3], m[4]];
			found.minutesFromNow = Number(count) * (/^(h|jam)/i.test(unit) ? 60 : 1);
			return true;
		},
	},
	// "every 25th (of the month)", "setiap tanggal 25"
	{
		pattern: new RegExp(`\\b(?:every|each)\\s+${ORDINAL}(?:\\s+of\\s+(?:the|every)\\s+month)?\\b|\\b(?:setiap|tiap)\\s+tanggal\\s+(\\d{1,2})\\b`, 'i'),
		read: (m, found) => {
			found.repeat = 'monthly';
			found.monthDay = Number(m[1] ?? m[2]);
			return found.monthDay >= 1 && found.monthDay <= 31;
		},
	},
	// "every Monday", "setiap (hari) Senin"; "setiap minggu" alone is every week
	{
		pattern: new RegExp(`\\b(?:every|each)\\s+(${ANY_DAY})s?\\b|\\b(?:setiap|tiap)\\s+(?:hari\\s+(${ANY_DAY})|(${FULL_DAY}))\\b`, 'i'),
		read: (m, found) => {
			found.repeat = 'weekly';
			found.weekday = WEEKDAYS[(m[1] ?? m[2] ?? m[3]).toLowerCase()];
			return true;
		},
	},
	{
		pattern: /\b(?:every\s*day|each\s+day|daily|setiap\s+hari|tiap\s+hari)\b/i,
		read: (_, found) => ((found.repeat = 'daily'), true),
	},
	{
		pattern: /\b(?:every\s+week|each\s+week|weekly|setiap\s+minggu|tiap\s+minggu|seminggu\s+sekali)\b/i,
		read: (_, found) => ((found.repeat = 'weekly'), true),
	},
	{
		pattern: /\b(?:every\s+month|each\s+month|monthly|setiap\s+bulan|tiap\s+bulan|sebulan\s+sekali)\b/i,
		read: (_, found) => ((found.repeat = 'monthly'), true),
	},
	// "on the 31st", "the 5th", "tanggal 25"
	{
		pattern: new RegExp(`\\b(?:on\\s+)?(?:the\\s+)?${ORDINAL}\\b|\\b(?:pada\\s+)?tanggal\\s+(\\d{1,2})\\b`, 'i'),
		read: (m, found) => {
			found.monthDay = Number(m[1] ?? m[2]);
			return found.monthDay >= 1 && found.monthDay <= 31;
		},
	},
	{ pattern: /\b(?:the\s+)?day\s+after\s+tomorrow\b|\blusa\b/i, read: (_, found) => ((found.dayOffset = 2), true) },
	{ pattern: /\btomorrow\b|\bbesok\b/i, read: (_, found) => ((found.dayOffset = 1), true) },
	{ pattern: /\btoday\b|\bhari\s+ini\b/i, read: (_, found) => ((found.dayOffset = 0), true) },
	// "on Friday", "next fri", "hari Minggu", or a day's full name on its own
	{
		pattern: new RegExp(`\\b(?:on|next|this|hari)\\s+(${ANY_DAY})\\b|\\b(${FULL_DAY})\\b`, 'i'),
		read: (m, found) => {
			found.weekday = WEEKDAYS[(m[1] ?? m[2]).toLowerCase()];
			return true;
		},
	},
	// "at 10am", "at 9", "jam 4 sore", "pukul 06.30"
	{
		pattern: new RegExp(`\\b(?:at|jam|pukul|pkl\\.?)\\s*(\\d{1,2})(?:[:.](\\d{2}))?(?:\\s*${PART})?(?![\\w.])`, 'i'),
		read: (m, found) => !!(found.time = clockTime(m[1], m[2], m[3])),
	},
	// "9:30", "06.30 pm"
	{
		pattern: new RegExp(`\\b(\\d{1,2})[:.](\\d{2})(?:\\s*${PART})?(?![\\w.])`, 'i'),
		read: (m, found) => !!(found.time = clockTime(m[1], m[2], m[3])),
	},
	// "6pm"
	{
		pattern: /\b(\d{1,2})\s*(am|pm|a\.m\.|p\.m\.)(?![\w.])/i,
		read: (m, found) => !!(found.time = clockTime(m[1], undefined, m[2])),
	},
];

const LEADING = /^(?:please\s+)?(?:remind\s+me\s+(?:to|about)|remind\s+me|(?:tolong\s+)?(?:ingatkan|ingetin)\s+(?:aku|saya|gue|gw)?(?:\s+(?:untuk|buat))?)\s+/i;
const DANGLING = /(?:^|\s)(?:at|on|every|each|the|of|to|untuk|buat|pada|jam|pukul|tiap|setiap)\s*$/i;

/** A calendar day as a date at UTC midnight, for counting days without any zone. */
const day = (date: string) => new Date(`${date}T00:00:00Z`);
const iso = (at: Date) => at.toISOString().slice(0, 10);
const addDays = (date: string, days: number) => iso(new Date(day(date).getTime() + days * 86_400_000));
const minutesOf = (time: string) => Number(time.slice(0, 2)) * 60 + Number(time.slice(3, 5));

/** The next date with this day of the month, from today (today itself while `time` is still ahead). */
function nextMonthDay(now: LocalNow, monthDay: number, time: string | undefined): string {
	const [year, month] = [Number(now.date.slice(0, 4)), Number(now.date.slice(5, 7)) - 1];
	for (let ahead = 0; ahead < 14; ahead++) {
		const candidate = new Date(Date.UTC(year, month + ahead, monthDay));
		if (candidate.getUTCDate() !== monthDay) continue;
		const date = iso(candidate);
		if (date > now.date || (date === now.date && (!time || minutesOf(time) > minutesOf(now.time)))) return date;
	}
	return now.date;
}

/** Reads one line into a reminder; what it can't read stays in the title. */
export function parseReminder(text: string, now: LocalNow): ParsedReminder {
	let rest = ` ${text.replace(/\s+/g, ' ').trim()} `;
	const found: Found = {};
	for (const rule of RULES) {
		const match = rule.pattern.exec(rest);
		if (match && rule.read(match, found)) rest = `${rest.slice(0, match.index)} ${rest.slice(match.index + match[0].length)}`;
	}

	let title = rest.replace(/\s+/g, ' ').trim().replace(LEADING, '');
	while (DANGLING.test(title)) title = title.replace(DANGLING, '').trim();
	title = title.replace(/^[\s,;:–-]+|[\s,;:–-]+$/g, '');
	const parsed: ParsedReminder = { title: title.charAt(0).toUpperCase() + title.slice(1) };

	let { time } = found;
	let date: string | undefined;
	if (found.minutesFromNow !== undefined) {
		const total = minutesOf(now.time) + found.minutesFromNow;
		date = addDays(now.date, Math.floor(total / 1440));
		time = `${String(Math.floor((total % 1440) / 60)).padStart(2, '0')}:${String(total % 60).padStart(2, '0')}`;
	} else if (found.dayOffset !== undefined) {
		date = addDays(now.date, found.dayOffset);
	} else if (found.weekday !== undefined) {
		let ahead = (found.weekday - day(now.date).getUTCDay() + 7) % 7;
		if (ahead === 0 && time && minutesOf(time) <= minutesOf(now.time)) ahead = 7;
		date = addDays(now.date, ahead);
	} else if (found.monthDay !== undefined) {
		date = nextMonthDay(now, found.monthDay, time);
	} else if (time) {
		date = minutesOf(time) > minutesOf(now.time) ? now.date : addDays(now.date, 1);
	}

	if (date) parsed.date = date;
	if (time) parsed.time = time;
	if (found.repeat) parsed.repeat = found.repeat;
	return parsed;
}

/** The local date-time the backend reads in the person's zone ("2026-10-25T10:00"), once both are known. */
export function dueAt(parsed: ParsedReminder): string | undefined {
	return parsed.date && parsed.time ? `${parsed.date}T${parsed.time}` : undefined;
}

/** Today's date and the time now, where the person is (UTC for a zone this runtime doesn't know). */
export function localNow(now: Date, timeZone?: string): LocalNow {
	let zone = timeZone ?? 'UTC';
	try {
		new Intl.DateTimeFormat('en', { timeZone: zone });
	} catch {
		zone = 'UTC';
	}
	const parts = Object.fromEntries(
		new Intl.DateTimeFormat('en-GB', { year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hourCycle: 'h23', timeZone: zone })
			.formatToParts(now)
			.map((part) => [part.type, part.value]),
	);
	return { date: `${parts.year}-${parts.month}-${parts.day}`, time: `${parts.hour}:${parts.minute}` };
}
