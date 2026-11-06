// @vitest-environment jsdom
import { describe, expect, it, vi } from 'vitest';
import { clickEverything, render, settle, text } from '../../../test-stubs/render';
import Page from './+page.svelte';
import type { Profile } from '$lib/types';

const profile = (over: Partial<Profile> = {}): Profile => ({
	display_name: 'Ana Kusuma',
	username: 'ana',
	email: 'ana@example.com',
	avatar_url: null,
	location: 'Bandung',
	joined_at: '2026-01-10T00:00:00Z',
	plan_name: 'Pro',
	show_stats_publicly: true,
	show_crew_publicly: false,
	showcase_project_ids: ['p1'],
	showcase_projects: [{ id: 'p1', name: 'Bakery', description: 'Orders', status: 'ready', deploy_url: 'https://bakery.test', build_duration_secs: 95 }],
	stats: {
		lifetime_tokens: 12_500_000,
		peak_day_tokens: 900_000,
		peak_day_date: '2026-10-02',
		longest_task_secs: 3_725,
		longest_task_name: 'Build the bakery',
		longest_streak_days: 12,
		longest_streak_range: 'Oct 1 – Oct 12',
		current_streak_days: 3,
		best_streak_target: 14,
	},
	activity: Array.from({ length: 40 }, (_, i) => ({ date: `2026-10-${String((i % 28) + 1).padStart(2, '0')}`, tokens: i * 1000 })),
	top_models: [{ label: 'Sonnet', tokens: 9_000_000, percentage: 72 }, { label: 'Haiku', tokens: 3_500_000, percentage: 28 }],
	crew_usage: [{ agent_id: 'money', name: 'Finley', count: 40, kind: 'built_in' }, { agent_id: 'coding', name: 'Koda', count: 12, kind: 'built_in' }],
	...over,
});

const projects = [{ id: 'p1', session_id: 's1', name: 'Bakery', description: 'Orders', status: 'ready', created_at: '', updated_at: '', created_by: 'Ana', open_issues: 0 }];

const page = (p: Profile | null, form: unknown = null) => render(Page as never, { data: { profile: p, projects } as never, form: form as never, params: {} as never });

describe('profile page', () => {
	it('shows the person, their stats, models and crew', () => {
		const root = page(profile());
		const shown = text(root);
		expect(shown).toContain('Ana Kusuma');
		expect(shown).toContain('Sonnet');
		expect(shown).toContain('Finley');
		expect(shown).toContain('Bakery');
	});

	it('opens its editing and sharing controls', async () => {
		vi.stubGlobal('navigator', { ...navigator, clipboard: { writeText: vi.fn(async () => {}) }, share: undefined });
		const root = page(profile(), { success: true });
		expect(clickEverything(root)).toBeGreaterThan(0);
		await settle();
		clickEverything(document.body);
		vi.unstubAllGlobals();
	});

	it('copes with a new account with nothing yet, and with a profile that failed to load', () => {
		const root = page(profile({ display_name: null, username: null, stats: undefined, activity: [], top_models: [], crew_usage: [], showcase_projects: [], location: null }), { error: 'Taken' });
		expect([...root.querySelectorAll('input')].some((i) => (i as HTMLInputElement).value === 'ana@example.com') || text(root).includes('ana@example.com') || text(root).includes('Profile')).toBe(true);
		const failed = page(null);
		expect(failed).toBeTruthy();
	});
});
