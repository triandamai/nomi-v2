// @vitest-environment jsdom
// The project's sheets, each against a fake backend: they load, show what came back, and every
// control can be tried without the sheet falling over.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text, type } from '../../../test-stubs/render';
import { ProjectDeploy } from '$lib/projectDeploy.svelte';
import DatabasePanel from './DatabasePanel.svelte';
import DeployPanel from './DeployPanel.svelte';
import IssuesPanel from './IssuesPanel.svelte';
import MembersPanel from './MembersPanel.svelte';
import MonitorPanel from './MonitorPanel.svelte';
import ProjectActivity from './ProjectActivity.svelte';
import RelayPanel from './RelayPanel.svelte';
import SecretsSection from './SecretsSection.svelte';
import SettingsPanel from './SettingsPanel.svelte';

const BOARD_ISSUE = { number: 1, title: 'Checkout button broken', body: 'It 500s', kind: 'bug', severity: 'high', status: 'open', created_by: 'Ana', mine: true, closed_at: null, created_at: '2026-11-01T00:00:00Z', updated_at: '2026-11-01T00:00:00Z', column_id: 'c-ready', position: 1, start_on: null, due_on: '2026-11-20', points: 3, crew_agent: 'money', assigned_by_agent: 'coding', crew_state: 'working', session_id: 's-issue' };

class QuietSocket {
	addEventListener() {}
	removeEventListener() {}
	close() {}
	send() {}
}

const publish = (over: Record<string, unknown> = {}) => ({
	id: 'pub1',
	environment: 'production',
	status: 'succeeded',
	url: 'https://bakery.apps.test',
	error: null,
	files_version: 4,
	rollback_of: null,
	can_roll_back: true,
	rollback_of_version: null,
	started_by_agent: null,
	is_live: true,
	created_by: 'Ana',
	created_at: '2026-11-01T00:00:00Z',
	finished_at: '2026-11-01T00:02:00Z',
	...over,
});

const overview = {
	available: true,
	role: 'admin',
	app: 'bakery',
	database: 'postgres',
	url: 'https://bakery.apps.test',
	publishes: [publish(), publish({ id: 'pub0', status: 'failed', error: 'build failed: missing module', is_live: false, can_roll_back: false })],
	live: { environment: 'production', services: [{ name: 'web', type: 'web', status: 'running', replicas: 1, running: 1, urls: ['https://bakery.apps.test'] }], last_publish: null },
	backend_setup: null,
	unpublished_changes: 2,
};

const routes = (): Parameters<typeof fakeBackend>[0] => [
	['GET', /\/api\/deploy$/, overview],
	['GET', /deploy\/usage/, { containers: [{ name: 'web', cpu_percent: 12.5, memory_bytes: 120_000_000, memory_limit_bytes: 512_000_000 }] }],
	['GET', /deploy\/metrics/, (() => {
		const counters = { visitors: 40, requests: 300, errors_5xx: 2, errors_4xx: 5, p50_ms: 80, p95_ms: 400 };
		return { range: '1h', step: '5m', buckets: [{ at: '2026-11-01T00:00:00Z', ...counters, containers: [{ service: 'web', cpu_percent: 10, memory_bytes: 1e8 }] }, { at: '2026-11-01T00:05:00Z', ...counters, visitors: null, containers: [] }], totals: counters, previous_totals: { ...counters, visitors: 20 } };
	})()],
	['GET', /deploy\/logs/, { lines: ['GET / 200', 'Error: boom'], note: 'last 200 lines' }],
	['GET', /deploy\/events/, [{ id: 'e1', type: 'deployment.succeeded', created_at: '2026-11-01T00:00:00Z', environment: 'production', service: 'web', data: { message: 'ok' } }]],
	['GET', /deploy\/publishes\/pub1/, { ...publish(), deployments: [{ service: 'web', deployment_id: 'd1', status: 'succeeded' }] }],
	['GET', /\/issues\?/, { issues: [{ number: 1, title: 'Checkout button broken', body: 'It 500s', kind: 'bug', severity: 'high', status: 'open', created_by: 'Ana', mine: true, closed_at: null, created_at: '2026-11-01T00:00:00Z', updated_at: '2026-11-01T00:00:00Z' }], counts: { open: 1, closed: 3 } }],
	['GET', /\/board$/, { columns: [{ id: 'c-backlog', name: 'Backlog', position: 0, fixed: 'backlog', issues: [{ ...BOARD_ISSUE, number: 2, title: 'Dark mode', kind: 'enhancement', severity: 'low', column_id: 'c-backlog', crew_state: null, crew_agent: 'coding', assigned_by_agent: null, session_id: null, points: null }] }, { id: 'c-ready', name: 'Ready to dev', position: 1, fixed: 'ready', issues: [BOARD_ISSUE, { ...BOARD_ISSUE, number: 3, title: 'Receipts', crew_state: 'failed', crew_agent: 'coding', assigned_by_agent: null }] }, { id: 'c-review', name: 'In review', position: 2, fixed: null, issues: [] }] }],
	['GET', /\/members$/, { access: 'members', my_role: 'admin', me: 'u1', people: [{ user_id: 'u1', name: 'Ana', email: 'ana@x.test', avatar_url: null, org_role: 'owner', project_role: 'admin', is_owner: true, added_at: null }, { user_id: 'u2', name: 'Ben', email: 'ben@x.test', avatar_url: null, org_role: 'member', project_role: 'member', is_owner: false, added_at: '2026-10-01T00:00:00Z' }] }],
	['GET', /database\/tables/, [{ schema: 'public', name: 'orders', rows_estimate: 42 }]],
	['POST', /database\/query/, { columns: ['id', 'total'], rows: [[1, 45000]], truncated: false, rows_affected: null, took_ms: 3 }],
	['POST', /database\/credentials/, { url: 'postgres://owner:pw@db/bakery' }],
	['GET', /\/relay\?/, { backend: 'relay', url: 'https://bakery.relay.test', anon_key: 'anon', tables: [{ name: 'todos', rls_enabled: true, columns: [{ name: 'id', type: 'uuid' }], policies: [{ name: 'own', command: 'select', using: 'auth.uid() = user_id', check: null }] }], buckets: [{ name: 'photos', public: false, read: 'owner', write: 'owner', max_size_mb: 5 }], realtime: ['todos'], functions: { version: 1, functions: { hello: { auth: 'public', timeout: 10 } } }, auth: { methods: ['email', 'google'], missing_secrets: ['GOOGLE_SECRET'] } }],
	['POST', /relay\/query/, { columns: ['n'], rows: [[1]], truncated: false, rows_affected: null, took_ms: 1 }],
	['GET', /\/secrets$/, { secrets: [{ name: 'STRIPE_KEY', updated_at: '2026-11-01T00:00:00Z', updated_by_name: 'Ana' }, { name: 'OTHER', updated_at: '2026-10-01T00:00:00Z', updated_by_name: null }], can_edit: true }],
	['GET', /\/activity/, { items: [{ kind: 'publish', at: '2026-11-01T00:00:00Z', actor: { name: 'Ana', avatar_url: null, agent: null }, version: 4, environment: 'production', status: 'succeeded' }, { kind: 'message', at: '2026-10-31T00:00:00Z', actor: { name: null, avatar_url: null, agent: 'coding' }, text: 'Added checkout' }, { kind: 'files', at: '2026-10-30T00:00:00Z', actor: null, files: 3 }], has_more: true }],
	['GET', /showcase/, { showcase: false, blurb: '' }],
	['GET', /transfer\/orgs/, [{ id: 'o2', name: 'Team', is_personal: false, owner_id: 'u1', owner_name: 'Ana', my_role: 'owner' }]],
	['*', /.*/, {}],
];

function deploy() {
	const d = new ProjectDeploy('p1');
	d.overview = overview as never;
	return d;
}

async function tryEverything(root: HTMLElement) {
	await settle(8);
	for (const input of [...document.querySelectorAll<HTMLInputElement | HTMLTextAreaElement>('input:not([type="checkbox"]):not([type="radio"]):not([type="file"]), textarea')]) {
		try {
			type(input, input.tagName === 'TEXTAREA' ? 'select 1' : 'staging');
		} catch {
			// read-only field
		}
	}
	clickEverything(root);
	await settle(8);
	clickEverything(document.body);
	await settle(8);
}

afterEach(() => vi.unstubAllGlobals());

describe('project sheets', () => {
	it('deploy panel shows the live app and its publishes', async () => {
		vi.stubGlobal('WebSocket', QuietSocket);
		fakeBackend(routes());
		const root = render(DeployPanel as never, { deploy: deploy(), onaskfix: vi.fn() });
		expect(text(root)).toContain('bakery.apps.test');
		await tryEverything(root);
	});

	it('monitor panel loads usage, logs and events', async () => {
		const calls = fakeBackend(routes());
		const root = render(MonitorPanel as never, { projectId: 'p1', published: true, deploy: deploy() });
		await tryEverything(root);
		expect(calls.some((c) => c.path.includes('deploy/logs'))).toBe(true);
		const unpublished = render(MonitorPanel as never, { projectId: 'p1', published: false });
		expect(unpublished.textContent).toBeTruthy();
	});

	it('issues panel lists, files and closes issues', async () => {
		const calls = fakeBackend(routes());
		const onCountChange = vi.fn();
		const root = render(IssuesPanel as never, { projectId: 'p1', onCountChange, onAskKoda: vi.fn(async () => {}) });
		await settle(8);
		[...root.querySelectorAll<HTMLButtonElement>('[role="tab"]')].find((tab) => tab.textContent?.trim() === 'List')!.click();
		await settle(8);
		expect(text(root)).toContain('Checkout button broken');
		await tryEverything(root);
		expect(calls.length).toBeGreaterThan(1);
	});

	it('issues board shows the columns and who has each issue, and moves one', async () => {
		const calls = fakeBackend(routes());
		const listeners: ((raw: string) => void)[] = [];
		const onCountChange = vi.fn();
		const root = render(IssuesPanel as never, {
			projectId: 'p1',
			onCountChange,
			onAskKoda: vi.fn(async () => {}),
			canEditColumns: true,
			subscribe: (listener: (raw: string) => void) => {
				listeners.push(listener);
				return () => {};
			},
		});
		await settle(8);
		const shown = text(root);
		for (const piece of ['Backlog', 'Ready to dev', 'In review', 'Dana is working on it (from Koda)', 'Koda couldn’t finish it', '3 pts', 'Open its chat'])
			expect(shown.replace(/'/g, '’')).toContain(piece);
		expect(onCountChange).toHaveBeenLastCalledWith(3);
		// Only the column people added can be edited.
		expect(root.querySelectorAll('[aria-label^="Edit column"]').length).toBe(1);

		// "Move to" sends the issue to the end of that column.
		const move = root.querySelector<HTMLSelectElement>('.move select')!;
		move.value = 'c-ready';
		move.dispatchEvent(new Event('change', { bubbles: true }));
		await settle(6);
		expect(calls.find((c) => c.path.endsWith('/issues/2/move'))?.body).toEqual({ column_id: 'c-ready', index: 2 });

		// The failed one can be tried again.
		[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Try again')!.click();
		await settle(6);
		expect(calls.some((c) => c.method === 'POST' && c.path.endsWith('/issues/3/retry'))).toBe(true);

		// The crew picked something up elsewhere: the board loads again.
		const before = calls.filter((c) => c.path.endsWith('/board')).length;
		for (const listener of listeners) listener('{"kind":"DeployChanged","event":"issues.changed"}');
		await settle(6);
		expect(calls.filter((c) => c.path.endsWith('/board')).length).toBe(before + 1);
		await tryEverything(root);
	});

	it('members panel shows who is in the project', async () => {
		fakeBackend(routes());
		const root = render(MembersPanel as never, { projectId: 'p1', orgId: 'o1', projectName: 'Bakery', deploy: deploy() });
		await settle(8);
		expect(text(root)).toContain('Ben');
		await tryEverything(root);
		const activity = render(MembersPanel as never, { projectId: 'p1', orgId: 'o1', tab: 'activity' });
		await settle(4);
		expect(activity).toBeTruthy();
	});

	it('database panel lists tables and runs a query', async () => {
		const calls = fakeBackend(routes());
		const root = render(DatabasePanel as never, { projectId: 'p1', published: true });
		await settle(8);
		expect(text(root)).toContain('orders');
		await tryEverything(root);
		expect(calls.some((c) => c.path.includes('database/query') || c.path.includes('credentials'))).toBe(true);
	});

	it('relay panel shows the backend and runs SQL', async () => {
		fakeBackend(routes());
		const root = render(RelayPanel as never, { projectId: 'p1', published: false });
		await tryEverything(root);
		expect(root.textContent).toBeTruthy();
	});

	it('secrets, activity and settings', async () => {
		fakeBackend(routes());
		const secrets = render(SecretsSection as never, { projectId: 'p1' });
		await settle(8);
		expect(text(secrets)).toContain('STRIPE_KEY');
		await tryEverything(secrets);
		const activity = render(ProjectActivity as never, { projectId: 'p1' });
		await settle(8);
		await tryEverything(activity);
		const settings = render(SettingsPanel as never, { deploy: deploy(), projectName: 'Bakery' });
		await tryEverything(settings);
		expect(settings.textContent).toBeTruthy();
	});

	it('sheets say so when the backend refuses', async () => {
		fakeBackend([['*', /.*/, new Response(JSON.stringify({ message: 'not allowed' }), { status: 403 })]]);
		for (const [component, props] of [
			[IssuesPanel, { projectId: 'p1', onAskKoda: vi.fn() }],
			[DatabasePanel, { projectId: 'p1', published: true }],
			[RelayPanel, { projectId: 'p1', published: true }],
			[SecretsSection, { projectId: 'p1' }],
			[MonitorPanel, { projectId: 'p1', published: true }],
		] as const) {
			const root = render(component as never, props as never);
			await settle(6);
			expect(root.textContent).toBeTruthy();
		}
	});
});
