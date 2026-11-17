// @vitest-environment jsdom
// Admin → Projects, Sandboxes and a project's issues.
import { describe, expect, it } from 'vitest';
import { render, text } from '../../../test-stubs/render';
import Projects from './projects/+page.svelte';
import ProjectIssues from './projects/[id]/issues/+page.svelte';
import Sandboxes from './sandboxes/+page.svelte';

const access = { projects: 'view' };
const page = (component: unknown, data: Record<string, unknown>) => render(component as never, { data: { access, ...data } as never, params: {} as never });
const link = (root: HTMLElement, href: string) => root.querySelector(`a[href="${href}"]`) as HTMLAnchorElement | null;

const project = (over: Record<string, unknown> = {}) => ({
	id: 'p1',
	name: 'roti-pagi',
	owner_email: 'budi@example.com',
	workspace: 'Roti Pagi',
	runtime: 'sandbox',
	created_at: '2026-11-15T10:00:00Z',
	sandbox_phase: 'running',
	sandbox_records: 3,
	open_issues: 1,
	issues: 4,
	...over,
});

describe('admin: projects', () => {
	it('shows each project’s sandbox and issues, each a way to the page about them', () => {
		const root = page(Projects, { projects: [project(), project({ id: 'p2', name: 'warung-kopi', sandbox_phase: null, sandbox_records: 0, open_issues: 0, issues: 0 })], q: '' });
		expect(text(root)).toContain('roti-pagi');
		expect(text(root)).toContain('budi@example.com');
		expect(link(root, '/admin/sandboxes?project=p1')?.textContent).toContain('1 active · 3 records');
		expect(link(root, '/admin/projects/p1/issues')?.textContent).toContain('1 open issue');
		expect(link(root, '/admin/sandboxes?project=p2')?.textContent).toContain('No sandbox');
		expect(link(root, '/admin/projects/p2/issues')?.textContent).toContain('No issues');
	});

	it('says so when the projects couldn’t be read', () => {
		expect(text(page(Projects, { projects: null, q: '' }))).toContain('The projects couldn’t be loaded.');
	});
});

describe('admin: sandboxes', () => {
	const record = (over: Record<string, unknown>) => ({ id: 'r1', project_id: 'p1', project_name: 'roti-pagi', provider: 'boat', sandbox_id: 'box-1', state: 'running', phase: 'running', current: true, created_at: '2026-11-15T10:00:00Z', updated_at: '2026-11-15T10:05:00Z', ended_at: null, ...over });

	it('lists every record whatever its state, and which is the live one', () => {
		const root = page(Sandboxes, {
			sandboxes: [record({}), record({ id: 'r2', sandbox_id: 'box-0', state: 'starting', phase: 'waking' }), record({ id: 'r3', sandbox_id: 'box-x', state: 'stopped', phase: 'stopped', current: false, ended_at: '2026-11-14T10:00:00Z' }), record({ id: 'r4', sandbox_id: 'box-y', state: 'gone', phase: 'idle', current: false })],
			project: 'p1',
		});
		for (const label of ['Running', 'Waking up', 'Stopped', 'Idle', 'box-1', 'box-x']) expect(text(root)).toContain(label);
		expect(text(root)).toContain('Live');
		expect(text(root)).toContain('Ended');
		expect(text(root)).toContain('Showing roti-pagi');
		expect(link(root, '/admin/sandboxes')).not.toBeNull();
	});

	it('has no filter to clear when it lists every project', () => {
		const root = page(Sandboxes, { sandboxes: [], project: null });
		expect(text(root)).toContain('No sandboxes yet.');
		expect(link(root, '/admin/sandboxes')).toBeNull();
	});
});

describe('admin: a project’s issues', () => {
	it('lists them read only, open and closed', () => {
		const root = page(ProjectIssues, {
			project: { id: 'p1', name: 'roti-pagi' },
			issues: [
				{ number: 2, title: 'Typo on the menu', kind: 'task', severity: 'low', status: 'closed', column_name: null, created_at: '2026-11-15T10:00:00Z', updated_at: '2026-11-15T11:00:00Z', closed_at: '2026-11-15T11:00:00Z' },
				{ number: 1, title: 'Checkout button does nothing', kind: 'bug', severity: 'high', status: 'open', column_name: 'Ready', created_at: '2026-11-14T10:00:00Z', updated_at: '2026-11-14T10:00:00Z', closed_at: null },
			],
		});
		expect(text(root)).toContain('roti-pagi');
		expect(text(root)).toContain('#1');
		expect(text(root)).toContain('Checkout button does nothing');
		expect(text(root)).toContain('Ready');
		expect(root.querySelector('button')).toBeNull();
		expect(link(root, '/admin/projects')).not.toBeNull();
	});
});
