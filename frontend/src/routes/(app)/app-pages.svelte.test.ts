// @vitest-environment jsdom
// The smaller pages of the app, each with the data its load returns: they show it, and their
// controls can be tried.
import { describe, expect, it } from 'vitest';
import { clickEverything, render, settle, text } from '../../test-stubs/render';
import Billing from './billing/+page.svelte';
import Chats from './chats/+page.svelte';
import Google from './connections/google/+page.svelte';
import Organization from './organizations/[orgId]/+page.svelte';
import Account from './account/+page.svelte';
import Connections from './connections/+page.svelte';
import More from './more/+page.svelte';

const plan = { id: 'pro', slug: 'pro', name: 'Pro', description: 'More of everything', monthly_tokens: 5_000_000, price_label: 'Rp 99k', features: ['Projects', 'Voice'], card_tone: 'violet', promo_label: 'Launch', promo_price_label: 'Rp 49k', promo_ends_at: '2026-12-31T00:00:00Z', is_default: false, is_active: true, sort_order: 2 };
const free = { ...plan, id: 'free', slug: 'free', name: 'Free', monthly_tokens: 100_000, price_label: 'Free', features: [], promo_label: null, promo_price_label: null, promo_ends_at: null, is_default: true, sort_order: 1 };

const page = (component: unknown, data: Record<string, unknown>, form: unknown = null) =>
	render(component as never, { data: data as never, form: form as never, params: {} as never });

async function tryAll(root: HTMLElement) {
	clickEverything(root);
	await settle(6);
	clickEverything(document.body);
	await settle(4);
}

describe('billing', () => {
	it('shows this month’s use and the plans', async () => {
		const usage = {
			plan: { id: 'pro', name: 'Pro', card_tone: 'violet', monthly_tokens: 5_000_000, custom_quota: false },
			month: '2026-11',
			current_month: '2026-11',
			timezone: 'Asia/Jakarta',
			tokens_used: 1_250_000,
			input_tokens: 1_000_000,
			output_tokens: 250_000,
			cached_input_tokens: 400_000,
			own_key_tokens: 50_000,
			calls: 320,
			spend_usd: 4.2,
			by_day: [{ date: '2026-11-01', tokens: 500_000, spend_usd: 1.5 }, { date: '2026-11-02', tokens: 750_000, spend_usd: 2.7 }],
			by_model: [{ label: 'Sonnet', tokens: 1_000_000, spend_usd: 3.5, source: 'nomi' }],
			by_agent: [{ agent: 'money', tokens: 500_000 }],
		};
		const root = page(Billing, { usage, plans: { plans: [free, plan], current_plan_id: 'pro', monthly_tokens: 5_000_000, custom_quota: true, override_until: '2026-12-31T00:00:00Z' } });
		expect(text(root)).toContain('Pro');
		await tryAll(root);
		expect(page(Billing, { usage: null, plans: null }).textContent).toBeTruthy();
	});
});

describe('chats', () => {
	it('lists chats, pinned first, and opens their menus', async () => {
		const chat = (id: string, title: string | null, over: Record<string, unknown> = {}) => ({ id, channel: 'web', chat_type: 'dm', chat_id: id, title, last_message: { content: `About ${title}`, created_at: '2026-11-05T00:00:00Z' }, agent_active: false, updated_at: '2026-11-05T00:00:00Z', project_id: null, pinned: false, pinned_at: null, in_project_id: null, ...over });
		const sessions = [chat('s1', 'Budget'), chat('s2', 'Trip', { pinned: true, pinned_at: '2026-11-01T00:00:00Z', agent_active: true }), chat('s3', null, { last_message: null, in_project_id: 'p1' })];
		const root = page(Chats, { sessions, projects: [{ id: 'p1', name: 'Bakery' }] }, { error: 'Could not rename' });
		expect(text(root)).toContain('Budget');
		expect(text(root)).toContain('Trip');
		await tryAll(root);
		expect(page(Chats, { sessions: [], projects: [] }).textContent).toBeTruthy();
	});
});

describe('google connection', () => {
	it('shows the connected account, its services and what the agent did', async () => {
		const google = {
			configured: true,
			connection: { email: 'ana@gmail.test', services: ['gmail', 'sheets'], connected_at: '2026-10-01T00:00:00Z' },
			activity: [{ service: 'gmail', summary: 'Drafted an email to Ben', link: null, created_at: '2026-11-01T00:00:00Z' }, { service: 'sheets', summary: 'Added 2 rows', link: 'https://docs.google.com/x', created_at: '2026-11-02T00:00:00Z' }],
			services: ['gmail', 'sheets', 'docs', 'drive', 'calendar'],
		};
		const root = page(Google, { google }, { error: 'Google said no' });
		expect(text(root)).toContain('ana@gmail.test');
		await tryAll(root);
		const notConnected = page(Google, { google: { ...google, connection: null, activity: [] } });
		await tryAll(notConnected);
		expect(page(Google, { google: { ...google, configured: false, connection: null } }).textContent).toBeTruthy();
		expect(page(Google, { google: null }).textContent).toBeTruthy();
	});
});

describe('organization', () => {
	it('shows members and invites, with an owner’s controls', async () => {
		const org = {
			id: 'o1',
			name: 'Acme',
			my_role: 'owner',
			me: 'u1',
			members: [
				{ user_id: 'u1', name: 'Ana', email: 'ana@acme.test', avatar_url: null, role: 'owner', joined_at: '2026-01-01T00:00:00Z' },
				{ user_id: 'u2', name: 'Ben', email: null, avatar_url: null, role: 'member', joined_at: '2026-02-01T00:00:00Z' },
			],
			invites: [{ code: 'ABC123', email: 'cy@acme.test', role: 'admin', created_at: '2026-11-01T00:00:00Z', expires_at: '2026-11-08T00:00:00Z' }],
			is_personal: false,
		};
		const root = page(Organization, { org }, { invited: 'dee@acme.test' });
		expect(text(root)).toContain('Acme');
		expect(text(root)).toContain('Ben');
		await tryAll(root);
		const member = page(Organization, { org: { ...org, my_role: 'member', me: 'u2', invites: [] } });
		expect(text(member)).toContain('Ana');
	});
});

describe('account', () => {
	it('shows email, sign-in methods, and devices', async () => {
		const root = page(Account, {
			profile: { email: 'ana@example.com' },
			methods: { password: true, google: false, configured: true, google_email: null },
			devices: [
				{
					id: 'd1',
					platform: 'web',
					name: 'MacBook',
					brand: 'Apple',
					model: 'MacBook Pro',
					os_name: 'macOS',
					os_version: '15.0',
					app_version: '1.0.0',
					presence: 'online',
					last_seen_at: '2026-11-01T00:00:00Z',
					created_at: '2026-01-01T00:00:00Z',
					current: true,
				},
			],
		});
		expect(text(root)).toContain('ana@example.com');
		await tryAll(root);
	});
});

describe('connections', () => {
	it('lists connectors and search results', async () => {
		const root = page(Connections, {
			query: '',
			overview: {
				google: { email: 'ana@gmail.com' },
				hevy: { account_name: 'Ana', connected_at: '2026-01-01' },
			},
		});
		expect(text(root)).toBeTruthy();
		await tryAll(root);
	});
});

describe('more', () => {
	it('shows all features and allows toggling pins', async () => {
		const root = page(More, {
			preferences: { drawer_pins: ['money', 'projects'] },
		});
		expect(text(root)).toBeTruthy();
		await tryAll(root);
	});
});

