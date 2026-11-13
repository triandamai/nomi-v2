import { m } from '$lib/paraglide/messages';
import type { ProfileDetails } from '$lib/adapters/api';
import { formatTokens } from '$lib/adapters/shell';
import { CREW, type CrewId } from '$lib/ds/agents/crew';

// Settings › Profile (App / Settings / Profile U9Gnrg): the person's numbers as the pen writes
// them, the year of token activity as a Monday-first grid, the models and crew they use, and the
// username rules the backend keeps (letters, numbers, underscores and periods, up to 32).

export type StatTile = { value: string; label: string; note: string; current?: boolean };
export type HeatDay = { date: string; tokens: number; level: 0 | 1 | 2 | 3 | 4 };
export type Heatmap = { weeks: (HeatDay | null)[][]; months: { label: string; week: number }[]; total: number };

const DAY = 86_400_000;
const intlLocale = (language: string) => (language === 'id' ? 'id-ID' : 'en-GB');
const utc = (date: string) => new Date(`${date}T00:00:00Z`);
const iso = (at: Date) => at.toISOString().slice(0, 10);

/** A month's short name as the pen writes it: "Sep", not British "Sept"; Indonesian's own. */
export function monthName(at: Date, language: string): string {
	return new Intl.DateTimeFormat(language === 'id' ? 'id-ID' : 'en-US', { month: 'short', timeZone: 'UTC' }).format(at);
}

/** "2 Aug" (and "14 Jul 2026" with the year) for a day, in the person's language. */
export function dayMonth(at: Date, language: string, year = false): string {
	return [at.getUTCDate(), monthName(at, language), year && at.getUTCFullYear()].filter(Boolean).join(' ');
}

const shortDate = (date: string, language: string) => dayMonth(utc(date), language);

const days = (count: number) => (count === 1 ? m.profile_day({ count }) : m.profile_days({ count }));

/** A task's length in mono: "38s", "4m 12s", "1h 5m". */
export function duration(secs: number): string {
	if (secs < 60) return `${secs}s`;
	if (secs < 3600) return `${Math.floor(secs / 60)}m ${secs % 60}s`;
	return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
}

/** The stats strip: lifetime tokens, the peak day, the longest task, the longest and current streaks. */
export function statTiles(stats: ProfileDetails['stats'], joinedAt: string, language: string): StatTile[] {
	const best = Math.max(stats.best_streak_target, stats.longest_streak_days);
	const left = best - stats.current_streak_days;
	return [
		{ value: formatTokens(stats.lifetime_tokens), label: m.profile_lifetime(), note: m.profile_since({ date: dayMonth(new Date(joinedAt), language) }) },
		{ value: formatTokens(stats.peak_day_tokens), label: m.profile_peak(), note: stats.peak_day_date ? shortDate(stats.peak_day_date, language) : m.profile_peak_none() },
		{ value: stats.longest_task_secs === null ? '—' : duration(stats.longest_task_secs), label: m.profile_longest_task(), note: stats.longest_task_name ?? m.profile_task_none() },
		{ value: days(stats.longest_streak_days), label: m.profile_longest_streak(), note: stats.longest_streak_range?.replace('-', '–') ?? m.profile_streak_none() },
		{
			value: days(stats.current_streak_days),
			label: m.profile_current_streak(),
			note: stats.current_streak_days === 0 ? m.profile_streak_start() : left > 0 ? m.profile_streak_left({ best, left }) : m.profile_streak_best(),
			current: true,
		},
	];
}

/**
 * The last 53 weeks of tokens as Monday-first columns, the last holding today (days after it are
 * null). Each day is shaded in four steps of the busiest one; months are named over the week their
 * first day falls in.
 */
export function heatmap(activity: { date: string; tokens: number }[], today: string, language: string, weeks = 53): Heatmap {
	const monday = new Date(utc(today).getTime() - ((utc(today).getUTCDay() + 6) % 7) * DAY);
	const start = new Date(monday.getTime() - (weeks - 1) * 7 * DAY);
	const tokens = new Map(activity.map((day) => [day.date, day.tokens]));
	const shown: number[] = [];
	const grid: (HeatDay | null)[][] = [];
	for (let week = 0; week < weeks; week++) {
		const column: (HeatDay | null)[] = [];
		for (let day = 0; day < 7; day++) {
			const date = iso(new Date(start.getTime() + (week * 7 + day) * DAY));
			if (date > today) {
				column.push(null);
				continue;
			}
			const count = tokens.get(date) ?? 0;
			shown.push(count);
			column.push({ date, tokens: count, level: 0 });
		}
		grid.push(column);
	}
	const most = Math.max(0, ...shown);
	for (const day of grid.flat()) if (day && day.tokens > 0) day.level = Math.min(4, Math.max(1, Math.ceil((4 * day.tokens) / most))) as HeatDay['level'];

	const months: Heatmap['months'] = [];
	grid.forEach((column, week) => {
		const first = column.find((day) => day?.date.endsWith('-01'));
		if (first) months.push({ label: monthName(utc(first.date), language), week });
	});
	return { weeks: grid, months, total: shown.reduce((sum, count) => sum + count, 0) };
}

/** The weekday with the most tokens ("Tue"), or null with none. */
export function mostActiveDay(activity: { date: string; tokens: number }[], language: string): string | null {
	const totals = Array.from({ length: 7 }, () => 0);
	for (const day of activity) totals[utc(day.date).getUTCDay()] += day.tokens;
	const most = Math.max(...totals);
	if (most <= 0) return null;
	return new Intl.DateTimeFormat(intlLocale(language), { weekday: 'short', timeZone: 'UTC' }).format(Date.UTC(2026, 9, 18 + totals.indexOf(most)));
}

/** Models you use most: "13.5K · 62%", the bar against the most used. */
export function modelUsage(models: ProfileDetails['top_models']): { label: string; value: string; share: number }[] {
	const most = Math.max(1, ...models.map((model) => model.tokens));
	return models.map((model) => ({ label: model.label, value: `${formatTokens(model.tokens)} · ${Math.round(model.percentage)}%`, share: model.tokens / most }));
}

/** Your crew: who they've worked with and how much (chats, or Koda's projects), busiest first. */
export function crewUsage(crew: ProfileDetails['crew_usage']): { agent: CrewId; name: string; count: string; share: number }[] {
	const known = new Set<string>(CREW.map((member) => member.id));
	const used = crew.filter((member) => member.count > 0 && known.has(member.agent_id));
	const most = Math.max(1, ...used.map((member) => member.count));
	return used
		.sort((a, b) => b.count - a.count)
		.map((member) => ({
			agent: member.agent_id as CrewId,
			name: member.name,
			count: member.kind === 'projects' ? m.profile_crew_projects({ count: member.count }) : String(member.count),
			share: member.count / most,
		}));
}

/** Under the name: "@mario · Jakarta · joined 14 Jul 2026 · Free plan". */
export function identityLine(profile: Pick<ProfileDetails, 'username' | 'location' | 'joined_at' | 'plan_name'>, language: string): string {
	const joined = dayMonth(new Date(profile.joined_at), language, true);
	return [profile.username && `@${profile.username}`, profile.location, m.profile_joined({ date: joined }), profile.plan_name].filter(Boolean).join(' · ');
}

/** What's wrong with a username before asking the backend, as it would say it; null when nothing is. */
export function usernameProblem(username: string): string | null {
	const name = username.trim().replace(/^@/, '');
	if (name.length > 32) return m.profile_username_long();
	if (!/^[A-Za-z0-9_.]*$/.test(name)) return m.profile_username_chars();
	return null;
}
