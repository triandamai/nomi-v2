// @vitest-environment jsdom
// The smaller admin pages, with what their loads return (or what they fetch themselves).
import { describe, expect, it } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text } from '../../../test-stubs/render';
import Agents from './agents/+page.svelte';
import Configuration from './configuration/+page.svelte';
import DynamicAgents from './dynamic-agents/+page.svelte';
import Embedding from './settings/embedding/+page.svelte';
import Notifications from './notifications/+page.svelte';
import Plans from './plans/+page.svelte';
import Support from './support/+page.svelte';
import Usage from './usage/+page.svelte';
import Xendit from './payment/xendit/+page.svelte';

const manage = { dashboard: 'manage', user: 'manage', plans: 'manage', notifications: 'manage', models: 'manage', tools: 'manage', agents: 'manage', support: 'manage', configuration: 'manage', integrations: 'manage' };
const plan = { id: 'pro', slug: 'pro', name: 'Pro', description: 'More', monthly_tokens: 5_000_000, price_label: 'Rp 99k', features: ['Projects'], card_tone: 'violet', promo_label: null, promo_price_label: null, promo_ends_at: null, is_default: false, is_active: true, sort_order: 2 };
const free = { ...plan, id: 'free', slug: 'free', name: 'Free', is_default: true, is_active: true, sort_order: 1 };

const page = (component: unknown, data: Record<string, unknown>, form: unknown = null) =>
	render(component as never, { data: { access: manage, ...data } as never, form: form as never, params: {} as never });

async function tryAll(root: HTMLElement) {
	await settle(6);
	clickEverything(root);
	await settle(6);
	clickEverything(document.body);
	await settle(4);
}

const action = (data: unknown) => ({ type: 'success', status: 200, data });

describe('admin pages', () => {
	it('usage: the month’s chat and embedding use by model', async () => {
		const usage = {
			month: '2026-11',
			current_month: '2026-11',
			chat: { tokens: 9_000_000, input_tokens: 7_000_000, output_tokens: 2_000_000, cached_input_tokens: 1_000_000, own_key_tokens: 500_000, calls: 4000, people: 30, spend_usd: 42.5 },
			embedding: { tokens: 300_000, calls: 900, estimated_calls: 12 },
			by_day: [{ date: '2026-11-01', chat_tokens: 4_000_000, embedding_tokens: 100_000 }, { date: '2026-11-02', chat_tokens: 5_000_000, embedding_tokens: 200_000 }],
			models: [
				{ admin_model_id: 'a1', label: 'Sonnet', deleted: false, provider: 'anthropic', model_id: 'sonnet', source: 'nomi', input_tokens: 6_000_000, output_tokens: 1_500_000, cached_input_tokens: 900_000, tokens: 7_500_000, calls: 3000, people: 25, spend_usd: 40 },
				{ admin_model_id: null, label: 'Own key', deleted: true, provider: 'openai', model_id: 'gpt', source: 'own_key', input_tokens: 1_000_000, output_tokens: 500_000, cached_input_tokens: 0, tokens: 1_500_000, calls: 1000, people: 5, spend_usd: 0 },
			],
			embedding_models: [{ provider: 'openai', model_id: 'text-embedding-3-small', tokens: 300_000, calls: 900, estimated_calls: 12, people: 30, current: true }],
		};
		const root = page(Usage, { usage });
		expect(text(root)).toContain('Sonnet');
		await tryAll(root);
		expect(page(Usage, { usage: null }).textContent).toBeTruthy();
	});

	it('plans: lists plans and opens the editor', async () => {
		fakeBackend([['*', /.*/, action({ success: true })]]);
		const root = page(Plans, { plans: [free, { ...plan, promo_label: 'Launch', promo_price_label: 'Rp 49k', promo_ends_at: '2026-12-31T00:00:00Z' }, { ...plan, id: 'old', name: 'Legacy', is_active: false }] }, { error: 'Slug taken' });
		expect(text(root)).toContain('Legacy');
		await tryAll(root);
	});

	it('custom agents: lists them and opens the editor', async () => {
		fakeBackend([['*', /.*/, action({ success: true })]]);
		const agent = { id: 'd1', name: 'Scout', system_prompt: 'Find things', intent_label: 'scouting', intent_description: 'research', granted_tools: ['list_transactions'], supports_todos: true, supports_plans: false, can_delegate: false, is_active: true, shape: 'circle', tone: 'sky', motion: 'bounce' };
		const root = page(DynamicAgents, { agents: [agent, { ...agent, id: 'd2', name: 'Off', is_active: false }], tools: ['list_transactions', 'set_budget'] }, { error: 'Label taken' });
		expect(text(root)).toContain('Scout');
		await tryAll(root);
	});

	it('running agents: people and their agents, with recent events', async () => {
		const agents = { users: [{ user_id: 'u1', label: 'ana@acme.test', agents: [{ agent_session_id: 'as1', session_id: 's1', agent_type: 'money', agent_display_name: 'Finley', channel: 'web', started_at: '2026-11-06T00:00:00Z', last_activity_at: '2026-11-06T00:01:00Z', current_phase: 'thinking', current_phase_detail: 'Adding it up' }] }] };
		const events = [{ id: 'e1', session_id: 's1', agent_session_id: 'as1', agent_type: 'money', agent_display_name: 'Finley', event_type: 'tool_call', created_at: '2026-11-06T00:01:00Z', tool_name: 'log_transaction', is_error: false }, { id: 'e2', session_id: null, agent_session_id: null, agent_type: null, agent_display_name: null, event_type: 'error', created_at: '2026-11-06T00:02:00Z', tool_name: null, is_error: true }];
		const root = page(Agents, { agents, events });
		expect(text(root)).toContain('Finley');
		await tryAll(root);
		expect(page(Agents, { agents: { users: [] }, events: [] }).textContent).toBeTruthy();
	});

	it('embedding settings and broadcast notifications', async () => {
		fakeBackend([['*', /.*/, action({ success: true, models: [{ id: 'text-embedding-3-small', label: null }] })]]);
		const embedding = page(Embedding, { settings: { provider: 'openai', model_id: 'text-embedding-3-small', base_url: null, api_key_masked: 'sk-…abcd' }, models: [] }, { error: 'Bad key' });
		expect(text(embedding)).toContain('text-embedding-3-small');
		await tryAll(embedding);
		expect(page(Embedding, { settings: null }).textContent).toBeTruthy();

		const sent = [{ id: 'b1', title: 'Maintenance tonight', body: 'Back by 2am', audience: 'all', plan_id: null, recipients: 120, created_at: '2026-11-01T00:00:00Z', sent_by: 'root@nomi.test' }];
		const notifications = page(Notifications, { sent, plans: [free, plan] }, { sent: 120 });
		expect(text(notifications)).toContain('Maintenance tonight');
		await tryAll(notifications);
	});

	it('support, configuration and Xendit load what they show themselves', async () => {
		const calls = fakeBackend(
			[
				['GET', /^support\/drafts/, [{ id: 'd1', question: 'How do I export?', title: 'Exporting', body: 'Use the export button', times_asked: 4, last_asked_at: '2026-11-01T00:00:00Z' }]],
				['GET', /^support\/articles/, [{ id: 'a1', title: 'Billing', body: 'How billing works', tags: ['billing'], published: true, updated_at: '2026-11-01T00:00:00Z' }]],
				['GET', /^support\/keys/, [{ id: 'k1', name: 'Website', prefix: 'sk_live_ab', product_name: 'Nomi', created_at: '2026-10-01T00:00:00Z', last_used_at: null, revoked_at: null }]],
				['GET', /^support\/questions/, [{ id: 'q1', question: 'Is it free?', answer: 'There is a free plan', answered: true, customer: 'cy@x.test', key_name: 'Website', created_at: '2026-11-02T00:00:00Z' }]],
				['GET', /^configuration/, { sections: [{ name: 'attachments', fields: [{ name: 'max_file_mb', env: 'ATTACHMENT_MAX_FILE_MB', kind: 'number', value: '10', set: true, source: 'saved' }] }, { name: 'storage', fields: [{ name: 'bucket', env: 'S3_BUCKET', kind: 'text', value: null, set: false, source: 'none' }, { name: 'secret_access_key', env: 'AWS_SECRET_ACCESS_KEY', kind: 'secret', value: null, set: true, source: 'environment' }] }] }],
				['GET', /^payment\/xendit$/, { provider: 'xendit', fields: [{ name: 'secret_key', env: 'XENDIT_SECRET_KEY', kind: 'secret', value: null, set: true, source: 'saved' }, { name: 'environment', env: null, kind: 'choice', choices: ['test', 'live'], value: 'test', set: true, source: 'saved' }], can_manage: true }],
				['POST', /^payment\/xendit\/test/, { success: true, message: 'Xendit client initialized successfully' }],
				['*', /.*/, { ok: true }],
			],
			{ strip: '/admin/api/' },
		);
		for (const component of [Support, Configuration, Xendit]) {
			const root = page(component, {});
			await tryAll(root);
			expect(root.textContent).toBeTruthy();
		}
		expect(calls.some((c) => c.path.startsWith('support/'))).toBe(true);
		expect(calls.some((c) => c.path === 'configuration')).toBe(true);
	});
});
