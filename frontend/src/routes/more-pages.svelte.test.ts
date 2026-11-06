// @vitest-environment jsdom
// More pages with the data their loads return: the landing page, a public profile, the design
// system, sign-up verification, and the app's preferences, chat, notifications and Hevy pages.
import { describe, expect, it } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text } from '../test-stubs/render';
import Welcome from './welcome/+page.svelte';
import PublicProfile from './@[username]/+page.svelte';
import DesignSystem from './design-system/+page.svelte';
import Verify from './verify/+page.svelte';
import Preferences from './(app)/preferences/+page.svelte';
import Chat from './(app)/chat/[sessionId]/+page.svelte';
import Notifications from './(app)/notifications/+page.svelte';
import Hevy from './(app)/connections/hevy/+page.svelte';

const page = (component: unknown, data: Record<string, unknown>, form: unknown = null) =>
	render(component as never, { data: data as never, form: form as never, params: { sessionId: 's1', username: 'ana' } as never });

async function tryAll(root: HTMLElement, skip?: RegExp) {
	await settle(4);
	clickEverything(root, skip);
	await settle(6);
	clickEverything(document.body, skip);
	await settle(4);
}

const preferences = { theme: 'system', accent_color: 'canopy', timezone: 'Asia/Jakarta', language: 'en', has_stored_timezone: true, drawer_pins: ['money', 'projects'], home_layout: null, animated_backdrop: true, drawer_recents: true };

describe('public pages', () => {
	it('welcome shows the showcase, signed in or not', async () => {
		const apps = [{ id: 'a1', name: 'Bakery', description: 'Orders', url: 'https://bakery.test', author: 'Ana', remixes: 3, thumbnail_url: null }];
		const root = page(Welcome, { apps, signedIn: false });
		expect(text(root)).toBeTruthy();
		await tryAll(root);
		expect(page(Welcome, { apps: [], signedIn: true }).textContent).toBeTruthy();
	});

	it('a public profile shows what its owner chose to share', async () => {
		const profile = {
			display_name: 'Ana K',
			username: 'ana',
			avatar_url: null,
			location: 'Bandung',
			joined_at: '2026-01-01T00:00:00Z',
			plan_name: 'Pro',
			show_stats_publicly: true,
			show_crew_publicly: true,
			stats: { lifetime_tokens: 1_000_000, peak_day_tokens: 50_000, peak_day_date: '2026-10-01', longest_task_secs: 600, longest_task_name: 'Build', longest_streak_days: 5, longest_streak_range: 'Oct 1 – 5', current_streak_days: 2, best_streak_target: 7 },
			activity: [{ date: '2026-10-01', tokens: 1000 }],
			top_models: [{ label: 'Sonnet', tokens: 900_000, percentage: 90 }],
			crew_usage: [{ agent_id: 'money', name: 'Finley', count: 4, kind: 'built_in' }],
			showcase_projects: [{ id: 'p1', name: 'Bakery', description: 'Orders', status: 'ready', deploy_url: 'https://bakery.test', build_duration_secs: 90 }],
		};
		const root = page(PublicProfile, { profile });
		expect(text(root)).toContain('Ana K');
		await tryAll(root);
		expect(page(PublicProfile, { profile: { ...profile, display_name: null, show_stats_publicly: false, show_crew_publicly: false, stats: null, showcase_projects: [] } }).textContent).toBeTruthy();
	});

	it('the design system page and sign-up verification render', async () => {
		const ds = page(DesignSystem, {});
		await tryAll(ds);
		expect(ds.textContent).toBeTruthy();
		const verify = page(Verify, { verification: { email: 'ana@acme.test' }, expired: false, isNew: true }, { error: 'Wrong code' });
		expect(text(verify)).toContain('ana@acme.test');
		await tryAll(verify);
		expect(page(Verify, { verification: null, expired: true, isNew: false }).textContent).toBeTruthy();
	});
});

describe('app pages', () => {
	it('preferences: theme, language, timezone and the drawer', async () => {
		fakeBackend([['*', /.*/, { type: 'success', status: 200, data: { success: true } }]]);
		const root = page(Preferences, { preferences }, { success: true });
		await tryAll(root);
		expect(root.textContent).toBeTruthy();
	});

	it('a chat page with its menu and personality', async () => {
		const message = { id: 'm1', sender: 'user', content: 'Hi', content_html: '<p>Hi</p>', content_blocks: null, created_at: '2026-11-05T00:00:00Z', my_feedback: null, agent_display_name: null, memory_count: 0 };
		const root = page(Chat, {
			messages: [message],
			hasOlder: false,
			personality: { versions: [{ version: 1, description: 'Warm', created_at: '2026-10-01T00:00:00Z' }] },
			chatPersonality: { chat: null, default: 'Warm' },
			agentActivity: [],
			agentStatus: null,
			title: 'Budget',
			thinkingLevel: 'low',
			modelOptions: { groups: [], current: null },
			sessionModel: null,
			chatMenu: { pinned: false, inProjectId: null, projects: [{ id: 'p1', name: 'Bakery', workspace: 'Acme' }] },
			preferences,
		});
		expect(text(root)).toContain('Budget');
		await tryAll(root, /delete|remove|model|provider|sign out/i);
	});

	it('notifications: the inbox by filter and email preferences', async () => {
		const items = [
			{ id: 'n1', kind: 'account', title: 'Welcome', body: 'Glad you are here', link: '/projects', read: false, created_at: '2026-11-05T00:00:00Z' },
			{ id: 'n2', kind: 'promo', title: 'Half price', body: 'This week', link: null, read: true, created_at: '2026-11-01T00:00:00Z' },
		];
		const inbox = { items, unread: 1, page: 1, pages: 2, total: 12, counts: { all: 12, unread: 1, read: 11, promo: 3 } };
		const root = page(Notifications, { inbox, preferences: { email_account: true, email_promos: false }, filter: 'all' });
		expect(text(root)).toContain('Welcome');
		await tryAll(root);
		expect(page(Notifications, { inbox: { ...inbox, items: [], total: 0 }, preferences: { email_account: false, email_promos: false }, filter: 'promo' }).textContent).toBeTruthy();
	});

	it('Hevy: connecting, connected, and a rejected key', async () => {
		const notConnected = page(Hevy, { loaded: true, connection: null, resume: 's1', keyRejected: true }, { error: 'key_rejected' });
		await tryAll(notConnected);
		const connected = page(Hevy, { loaded: true, connection: { provider: 'hevy', account_name: 'ana_lifts', connected_at: '2026-10-01T00:00:00Z', last_used_at: '2026-11-01T00:00:00Z' }, resume: null, keyRejected: false }, { connected: true });
		expect(text(connected)).toContain('ana_lifts');
		await tryAll(connected);
		expect(page(Hevy, { loaded: false, connection: null, resume: null, keyRejected: false }).textContent).toBeTruthy();
	});
});
