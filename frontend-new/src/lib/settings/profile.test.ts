import { describe, expect, it } from 'vitest';
import { crewUsage, dayMonth, duration, heatmap, identityLine, modelUsage, mostActiveDay, statTiles, usernameProblem } from './profile';

const stats = {
	lifetime_tokens: 186_400,
	peak_day_tokens: 14_200,
	peak_day_date: '2026-08-02',
	longest_task_secs: 252,
	longest_task_name: 'salah-daily-tracker',
	longest_streak_days: 9,
	longest_streak_range: '1-9 Aug',
	current_streak_days: 3,
	best_streak_target: 9,
};

describe('the profile’s numbers', () => {
	it('are five tiles: lifetime tokens, the peak day, the longest task and both streaks', () => {
		expect(statTiles(stats, '2026-07-14T03:00:00Z', 'en')).toEqual([
			{ value: '186.4K', label: 'Lifetime tokens', note: 'since 14 Jul' },
			{ value: '14.2K', label: 'Peak day', note: '2 Aug' },
			{ value: '4m 12s', label: 'Longest task', note: 'salah-daily-tracker' },
			{ value: '9 days', label: 'Longest streak', note: '1–9 Aug' },
			{ value: '3 days', label: 'Current streak', note: 'best is 9 — 6 to go', current: true },
		]);
	});

	it('say plainly when there’s nothing yet, and when the streak is the best one', () => {
		const none = statTiles({ ...stats, peak_day_tokens: 0, peak_day_date: null, longest_task_secs: null, longest_task_name: null, longest_streak_days: 0, longest_streak_range: null, current_streak_days: 0, best_streak_target: 1 }, '2026-07-14T03:00:00Z', 'en');
		expect(none.map((tile) => [tile.value, tile.note])).toEqual([
			['186.4K', 'since 14 Jul'],
			['0', 'no busy day yet'],
			['—', 'nothing built yet'],
			['0 days', 'no streak yet'],
			['0 days', 'use Nomi today to start one'],
		]);
		expect(statTiles({ ...stats, current_streak_days: 9 }, '2026-07-14T03:00:00Z', 'en')[4].note).toBe('your best yet');
		expect(statTiles({ ...stats, current_streak_days: 1, longest_streak_days: 1 }, '2026-07-14T03:00:00Z', 'en').slice(3).map((tile) => tile.value)).toEqual(['1 day', '1 day']);
	});

	it('write dates as the pen does: "2 Sep", not British "Sept"; Indonesian in its own words', () => {
		expect(dayMonth(new Date('2026-09-02T05:00:00Z'), 'en')).toBe('2 Sep');
		expect(dayMonth(new Date('2026-08-17T05:00:00Z'), 'id', true)).toBe('17 Agu 2026');
	});

	it('write a task’s length in mono: seconds, minutes and seconds, hours and minutes', () => {
		expect([38, 252, 3900].map(duration)).toEqual(['38s', '4m 12s', '1h 5m']);
	});
});

describe('the token heatmap', () => {
	// Wed 21 Oct 2026.
	const map = heatmap(
		[
			{ date: '2026-10-21', tokens: 400 },
			{ date: '2026-10-19', tokens: 100 },
			{ date: '2026-08-02', tokens: 14_200 },
			{ date: '2025-10-01', tokens: 999 },
		],
		'2026-10-21',
		'en',
	);

	it('is 53 weeks of Monday-first columns, the last holding today and nothing after it', () => {
		expect(map.weeks).toHaveLength(53);
		expect(map.weeks.every((week) => week.length === 7)).toBe(true);
		const last = map.weeks[52];
		expect(last.map((day) => day?.date ?? null)).toEqual(['2026-10-19', '2026-10-20', '2026-10-21', null, null, null, null]);
		expect(map.weeks[0][0]?.date).toBe('2025-10-20');
	});

	it('shades each day in four steps of the busiest one, leaving days off the grid out of the total', () => {
		const last = map.weeks[52];
		expect([last[0]?.level, last[1]?.level, last[2]?.level]).toEqual([1, 0, 1]);
		const peak = map.weeks.flat().find((day) => day?.date === '2026-08-02');
		expect(peak?.level).toBe(4);
		expect(map.total).toBe(14_700);
	});

	it('names each month over the week its first day falls in', () => {
		expect(map.months[0]).toEqual({ label: 'Nov', week: 1 });
		expect(map.months.at(-1)).toEqual({ label: 'Oct', week: 49 });
		expect(map.months).toHaveLength(12);
	});

	it('finds the weekday with the most tokens', () => {
		expect(mostActiveDay([{ date: '2026-10-20', tokens: 5 }, { date: '2026-10-13', tokens: 7 }, { date: '2026-10-21', tokens: 9 }], 'en')).toBe('Tue');
		expect(mostActiveDay([], 'en')).toBeNull();
	});
});

describe('what the person uses', () => {
	it('lists the models with their tokens and share, the bar against the most used', () => {
		expect(modelUsage([{ label: 'deepseek-flash', tokens: 13_500, percentage: 62 }, { label: 'gemini-flash', tokens: 5_200, percentage: 24 }])).toEqual([
			{ label: 'deepseek-flash', value: '13.5K · 62%', share: 1 },
			{ label: 'gemini-flash', value: '5.2K · 24%', share: 5_200 / 13_500 },
		]);
	});

	it('lists the crew they work with, leaving out who they haven’t', () => {
		const rows = crewUsage([
			{ agent_id: 'nomi', name: 'Nomi', count: 142, kind: 'chats' },
			{ agent_id: 'dana', name: 'Dana', count: 31, kind: 'chats' },
			{ agent_id: 'kala', name: 'Kala', count: 0, kind: 'chats' },
			{ agent_id: 'koda', name: 'Koda', count: 2, kind: 'projects' },
		]);
		expect(rows).toEqual([
			{ agent: 'nomi', name: 'Nomi', count: '142', share: 1 },
			{ agent: 'dana', name: 'Dana', count: '31', share: 31 / 142 },
			{ agent: 'koda', name: 'Koda', count: '2 projects', share: 2 / 142 },
		]);
	});
});

describe('the profile’s name line and username', () => {
	it('puts the username, place, when they joined and their plan in one mono line', () => {
		expect(identityLine({ username: 'mario', location: 'Jakarta', joined_at: '2026-07-14T03:00:00Z', plan_name: 'Free plan' }, 'en')).toBe('@mario · Jakarta · joined 14 Jul 2026 · Free plan');
		expect(identityLine({ username: null, location: null, joined_at: '2026-07-14T03:00:00Z', plan_name: 'Pro plan' }, 'en')).toBe('joined 14 Jul 2026 · Pro plan');
	});

	it('takes what the backend takes: letters, numbers, underscores and periods, up to 32', () => {
		expect(usernameProblem('mario_p.2')).toBeNull();
		expect(usernameProblem('@mario')).toBeNull();
		expect(usernameProblem('mario-p')).toBe('Letters, numbers, underscores and periods only.');
		expect(usernameProblem('x'.repeat(33))).toBe('At most 32 characters.');
	});
});
