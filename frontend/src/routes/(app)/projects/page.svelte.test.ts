// @vitest-environment jsdom
import { describe, expect, it } from 'vitest';
import { click, controls, render, text, type } from '../../../test-stubs/render';
import Page from './+page.svelte';
import { m } from '$lib/paraglide/messages';
import type { ProjectSummary } from '$lib/types';

const project = (over: Partial<ProjectSummary>): ProjectSummary => ({
	id: 'p1aaaaaa',
	session_id: 's1',
	name: 'Bakery',
	description: 'Orders for the bakery',
	status: 'ready',
	created_at: '2026-10-01T00:00:00Z',
	updated_at: '2026-11-01T00:00:00Z',
	created_by: 'Ana',
	open_issues: 0,
	stack: 'sveltekit',
	chats_count: 2,
	deploys_count: 3,
	files_count: 12,
	deploy_url: 'https://bakery.apps.test',
	...over,
});

const projects = [
	project({}),
	project({ id: 'p2bbbbbb', session_id: 's2', name: 'Todo', description: null, status: 'building', updated_at: '2026-11-05T00:00:00Z' }),
	project({ id: 'p3cccccc', session_id: 's3', name: 'Shop', status: 'ready', open_issues: 2, deploy_url: null }),
];

const data = (over: Record<string, unknown> = {}) => ({
	orgs: [{ id: 'o1', name: 'Acme', role: 'owner', members: 2, projects: 3 }, { id: 'o2', name: 'Side', role: 'member', members: 1, projects: 0 }],
	orgId: 'o1',
	projects,
	sessions: [{ id: 's9', title: 'Logo ideas', project_id: 'p1aaaaaa', updated_at: '2026-11-02T00:00:00Z' }],
	members: [{ user_id: 'u1', name: 'Ana', role: 'owner' }, { user_id: 'u2', name: 'Ben', email: 'ben@x.test', role: 'member' }],
	...over,
});

const page = (d = data(), form: unknown = null) => render(Page as never, { data: d as never, form: form as never, params: {} as never });

describe('projects page', () => {
	it('lists the organization’s projects with their state', () => {
		const root = page();
		const shown = text(root);
		for (const name of ['Bakery', 'Todo', 'Shop']) expect(shown).toContain(name);
	});

	it('filters by state and search, and sorts', () => {
		const root = page();
		const search = root.querySelector<HTMLInputElement>(`input[placeholder="${m.projects_search()}"]`);
		type(search, 'todo');
		expect(text(root)).toContain('Todo');
		type(search, 'zzz');
		type(search, '');
		for (const label of [/live/i, /building/i, /attention/i, /all/i]) {
			for (const control of controls(label, root)) click(control);
		}
		for (const control of controls(/name|recent|sort/i, root)) click(control);
		expect(text(root)).toContain('Bakery');
	});

	it('opens a project’s details', () => {
		const root = page();
		const card = [...root.querySelectorAll<HTMLElement>('[onclick], button, [role="button"], article, li, div')].reverse().find((el) => el.textContent?.trim().startsWith('Shop'));
		click(card);
		expect(text()).toContain('Shop');
	});

	it('asks where projects live before anything else', () => {
		const root = page(data({ orgId: null, projects: [], members: [] }), { orgError: 'Name it' });
		expect(root.textContent).toBeTruthy();
	});

	it('shows an empty organization', () => {
		const root = page(data({ projects: [], sessions: [] }));
		expect(root.textContent).toBeTruthy();
		for (const control of controls(/new|import|create/i, root).slice(0, 3)) click(control);
	});
});
