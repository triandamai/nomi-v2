// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { m } from '$lib/paraglide/messages';
import { click, control, controls, fakeBackend, render, settle, text } from '../../../../test-stubs/render';
import Page from './+page.svelte';

const status = {
	configured: true,
	can_reveal: true,
	webhook_url: 'https://shipyard.dev/hooks/nomi',
	webhook_registered: true,
	webhook_enabled: true,
	webhook_target: 'https://nomi.app/api/shipyard/webhook',
	webhook_from_env: false,
	registered_at: '2026-10-01T00:00:00Z',
	last_event_at: '2026-11-01T00:00:00Z',
};
const endpoint = { id: 'e1', url: 'https://nomi.app/api/shipyard/webhook', events: ['deploy.finished'], enabled: true, org: null, created_at: '2026-10-01T00:00:00Z' };
const other = { ...endpoint, id: 'e2', url: 'https://other.dev/hook', enabled: false, org: 'acme' };
const delivery = { id: 'd1', event_id: 'ev1', event_type: 'deploy.finished', status: 'failed', attempts: 3, last_status_code: 500, last_error: 'boom', created_at: '2026-11-01T00:00:00Z', delivered_at: null };
const org = { id: 'o1', name: 'Acme', synced_at: '2026-11-01T00:00:00Z', projects: 2, published: 1 };
const app = { org_id: 'o1', org_name: 'Acme', key: 'shop', name: 'Shop', environments: ['production', 'staging'], project: { id: 'p1', name: 'Shop', session_id: 's1' } };
const orphan = { ...app, key: 'blog', name: 'Blog', project: null };
const detail = {
	key: 'shop',
	dev_database: true,
	environments: [
		{ environment: 'production', services: [{ name: 'web', type: 'web', status: 'running', replicas: 1, running: 1, urls: ['https://shop.app'] }], last_publish: { status: 'succeeded', urls: ['https://shop.app'], created_at: '2026-11-01T00:00:00Z' } },
		{ environment: 'staging', services: [], last_publish: null },
	],
};
const secret = { ref: 'sec1', name: 'API_KEY', kind: 'user', environment: 'production', used_by: ['web'], updated_at: '2026-11-01T00:00:00Z' };
const event = { id: 'x1', type: 'deploy.finished', project_name: 'Shop', environment: 'production', service: 'web', created_at: '2026-11-01T00:00:00Z', received_at: '2026-11-01T00:00:01Z' };

function backend() {
	return fakeBackend(
		[
			['GET', 'shipyard', status],
			['POST', 'shipyard/webhook', status],
			['POST', 'shipyard/events/catch-up', { orgs: 1, new_events: 4, errors: ['org x unreachable'] }],
			['GET', 'shipyard/webhooks', { endpoints: [endpoint, other], own_id: 'e1' }],
			['GET', /^shipyard\/webhooks\/e1\/deliveries/, [delivery]],
			['POST', 'shipyard/webhooks/e1/rotate', { secret: 'whsec_new' }],
			['*', /^shipyard\/webhooks\/e\d/, {}],
			['POST', 'shipyard/deliveries/d1/redeliver', {}],
			['GET', 'shipyard/orgs', [org, { ...org, id: 'o2', name: 'Beta', synced_at: null }]],
			['GET', /secrets$/, [secret]],
			['GET', 'shipyard/orgs/o1/apps/shop', detail],
			['*', /^shipyard\/orgs\/o1\/apps\//, {}],
			['*', /^shipyard\/orgs\//, {}],
			['GET', 'shipyard/apps', { apps: [app, orphan], candidates: [{ org_id: 'o1', projects: [{ id: 'p9', name: 'Blog project' }] }], errors: ['one org failed'] }],
			['POST', 'shipyard/apps/link', {}],
			['GET', /secrets$/, [secret]],
			['GET', 'shipyard/orgs/o1/apps/shop', detail],
			['*', /^shipyard\/apps\//, {}],
			['GET', /^shipyard\/events/, { events: [event], total: 1, page: 1, per_page: 20 }],
		],
		{ strip: '/admin/api/' },
	);
}

const page = (access: Record<string, string>) => render(Page as never, { data: { access } as never, params: {} as never });

async function openTab(label: string) {
	click(control(label));
	await settle();
}

describe('admin integrations', () => {
	it('shows the Shipyard connection on the overview', async () => {
		const calls = backend();
		const root = page({ integrations: 'manage' });
		await settle();
		expect(calls[0]).toMatchObject({ method: 'GET', path: 'shipyard' });
		expect(text(root)).toContain('https://shipyard.dev/hooks/nomi');
	});

	it('catches up on missed events and shows the errors that came back', async () => {
		const calls = backend();
		const root = page({ integrations: 'manage' });
		await settle();
		click(control(m.integ_catch_up()));
		await settle();
		expect(calls.some((c) => c.method === 'POST' && c.path === 'shipyard/events/catch-up')).toBe(true);
		expect(text(root)).toContain('org x unreachable');
	});

	it('manages webhook endpoints: deliveries, test, rotate, redeliver', async () => {
		const calls = backend();
		const root = page({ integrations: 'manage' });
		await settle();
		await openTab(m.integ_tab_webhooks());
		expect(text(root)).toContain('https://other.dev/hook');

		click(controls(m.integ_deliveries())[0]);
		await settle();
		expect(text(root)).toContain('boom');
		click(control(m.integ_redeliver()));
		await settle();
		expect(calls.some((c) => c.path === 'shipyard/deliveries/d1/redeliver')).toBe(true);

		click(controls(m.integ_test())[0]);
		await settle();
		expect(calls.some((c) => c.path === 'shipyard/webhooks/e1/test')).toBe(true);

		click(controls(m.integ_rotate())[0]);
		click(control(m.integ_confirm()));
		await settle();
		expect(text(root)).toContain('whsec_new');
		click(control(m.integ_done()));

		click(controls(m.common_delete())[1]);
		click(control(m.integ_confirm()));
		await settle();
		expect(calls.some((c) => c.method === 'DELETE' && c.path === 'shipyard/webhooks/e2')).toBe(true);

		click(control(m.integ_add_endpoint()));
		expect(document.querySelectorAll('form').length).toBeGreaterThan(0);
	});

	it('hides webhook actions from someone who can only view', async () => {
		backend();
		page({ integrations: 'view' });
		await settle();
		await openTab(m.integ_tab_webhooks());
		expect(controls(m.integ_rotate())).toHaveLength(0);
		expect(controls(m.integ_test())).toHaveLength(0);
		expect(controls(m.integ_catch_up())).toHaveLength(0);
	});

	it('syncs and removes organizations', async () => {
		const calls = backend();
		const root = page({ integrations: 'manage' });
		await settle();
		await openTab(m.integ_tab_orgs());
		expect(text(root)).toContain('Beta');
		click(control(m.integ_sync()));
		await settle();
		expect(calls.some((c) => c.method === 'POST' && c.path === 'shipyard/orgs/o1/sync')).toBe(true);
		click(control(m.integ_remove()));
		click(control(m.integ_confirm()));
		await settle();
		expect(calls.some((c) => c.method === 'DELETE' && c.path === 'shipyard/orgs/o1')).toBe(true);
	});

	it('opens an app with its environments and secrets', async () => {
		const calls = backend();
		const root = page({ integrations: 'manage' });
		await settle();
		await openTab(m.integ_tab_apps());
		expect(text(root)).toContain('one org failed');
		click(controls(m.integ_open_app())[0]);
		await settle();
		expect(text()).toContain('API_KEY');
		expect(text()).toContain('https://shop.app');
		click(control(m.integ_take_down()));
		click(control(m.integ_confirm()));
		await settle();
		expect(calls.some((c) => c.method === 'DELETE' && /envs\/production$/.test(c.path))).toBe(true);
		click(controls(m.common_delete()).at(-1));
		click(control(m.integ_confirm()));
		await settle();
		expect(calls.some((c) => c.method === 'DELETE' && c.path.includes('secrets/API_KEY'))).toBe(true);
		click(control(m.integ_terminate()));
		click(control(m.integ_confirm()));
		await settle();
		expect(calls.some((c) => c.method === 'DELETE' && c.path === 'shipyard/orgs/o1/apps/shop')).toBe(true);
	});

	it('lists the events received', async () => {
		backend();
		const root = page({ integrations: 'manage' });
		await settle();
		await openTab(m.integ_tab_events());
		expect(text(root)).toContain('deploy.finished');
	});

	it('shows the API error when Shipyard cannot be read', async () => {
		fakeBackend([['GET', 'shipyard', new Response('shipyard down', { status: 502 })]], { strip: '/admin/api/' });
		const root = page({ integrations: 'manage' });
		await settle();
		expect(text(root)).toContain('shipyard down');
	});
});
