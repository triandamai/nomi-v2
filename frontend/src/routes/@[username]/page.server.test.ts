import { describe, expect, it, vi } from 'vitest';
import { load } from './+page.server';

vi.mock('$lib/server/api', () => ({
	apiFetch: vi.fn(async (_fetch, _cookies, path) => {
		if (path === '/api/public/profile/mario') {
			return {
				ok: true,
				status: 200,
				json: async () => ({
					display_name: 'Mario Prasetyo',
					username: 'mario',
					avatar_url: null,
					location: 'Jakarta',
					joined_at: '2026-07-14T00:00:00Z',
					plan_name: 'Free plan',
					show_stats_publicly: true,
					show_crew_publicly: false,
					stats: {
						lifetime_tokens: 186400,
						peak_day_tokens: 14200,
						peak_day_date: '2 Aug',
						longest_task_secs: 252,
						longest_task_name: 'salah tracker',
						longest_streak_days: 9,
						longest_streak_range: '1-9 Aug',
						current_streak_days: 3,
						best_streak_target: 9,
					},
					activity: [],
					top_models: [],
					crew_usage: null,
					showcase_projects: [
						{
							id: '11111111-1111-1111-1111-111111111111',
							name: 'salah-daily-tracker',
							description: 'Daily prayer tracker with streaks',
							status: 'ready',
							deploy_url: 'https://salah-tracker.trian.space',
							build_duration_secs: 252,
						},
					],
				}),
			};
		}
		return { ok: false, status: 404, text: async () => 'Not found' };
	}),
}));

describe('public profile page.server', () => {
	it('loads public profile data for a username', async () => {
		const result = (await load({
			params: { username: '@mario' },
			cookies: {} as any,
			fetch: vi.fn() as any,
		} as any)) as { profile: any };

		expect(result.profile).toBeDefined();
		expect(result.profile.username).toBe('mario');
		expect(result.profile.display_name).toBe('Mario Prasetyo');
		expect(result.profile.showcase_projects).toHaveLength(1);
		expect(result.profile.showcase_projects[0].name).toBe('salah-daily-tracker');
		expect(result.profile.stats.lifetime_tokens).toBe(186400);
	});
});
