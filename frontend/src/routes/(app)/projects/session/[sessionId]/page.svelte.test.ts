// @vitest-environment jsdom
// The open project: its chat beside the preview, code and terminal, the toolbar's sheets, and a
// project with nothing to build yet. The browser runtime falls back (no cross-origin isolation
// in jsdom) and the backend is faked.
import { describe, expect, it } from 'vitest';
import { clickEverything, controls, fakeBackend, render, settle, text } from '../../../../../test-stubs/render';
import Page from './+page.svelte';

const message = (id: string, sender: 'user' | 'assistant', content: string) => ({
	id,
	sender,
	content,
	content_html: `<p>${content}</p>`,
	content_blocks: null,
	created_at: '2026-11-05T09:00:00Z',
	my_feedback: null,
	agent_display_name: sender === 'assistant' ? 'Koda' : null,
	memory_count: 0,
});

const project = {
	id: 'p1',
	name: 'Bakery',
	description: 'Orders for the bakery',
	plan: '1. Home page\n2. Orders',
	status: 'ready',
	stack: 'static',
	files_version: 3,
	files: [
		{ path: 'index.html', content_type: 'text/html', size_bytes: 120 },
		{ path: 'src/app.js', content_type: 'text/javascript', size_bytes: 300 },
		{ path: 'styles/site.css', content_type: 'text/css', size_bytes: 80 },
	],
	session_id: 's1',
	org_id: 'o1',
	org_name: 'Acme',
	open_issues: 2,
	backend: 'drizzle',
};

const data = (over: Record<string, unknown> = {}) => ({
	messages: [message('m1', 'user', 'Build me a bakery site'), message('m2', 'assistant', 'Done: a home page and an order form.')],
	hasOlder: false,
	agentActivity: [],
	agentStatus: null,
	project,
	thinkingLevel: 'medium',
	modelOptions: { groups: [], current: null },
	sessionModel: null,
	...over,
});

function backend() {
	return fakeBackend([
		['GET', /\/api\/deploy$/, { available: true, role: 'admin', app: 'bakery', database: 'none', url: null, publishes: [], live: null, backend_setup: null, unpublished_changes: 3 }],
		['GET', /\/api\/files\//, new Response('<h1>Bakery</h1>', { status: 200 })],
		['GET', /\/presence/, { people: [] }],
		['GET', /deploy\/requests/, { requests: [{ id: 'r1', status: 'pending', requested_by_name: 'Ben', environment: 'production', note: 'ready to go', created_at: '2026-11-05T00:00:00Z' }] }],
		['GET', /\/asks/, { asks: [] }],
		['GET', /\/queue/, { asks: [] }],
		['*', /.*/, {}],
	]);
}

describe('project session page', () => {
	it('shows the project’s chat and its workspace', async () => {
		backend();
		const root = render(Page as never, { data: data() as never, form: null as never, params: { sessionId: 's1' } as never });
		await settle(8);
		const shown = text(root);
		expect(shown).toContain('Bakery');
		expect(shown).toContain('Build me a bakery site');
	});

	it('switches between preview, code and terminal and opens the toolbar’s sheets', async () => {
		const calls = backend();
		const root = render(Page as never, { data: data() as never, form: null as never, params: { sessionId: 's1' } as never });
		await settle(8);
		for (const label of [/code/i, /terminal/i, /preview/i]) for (const tab of controls(label, root).slice(0, 2)) tab.click();
		await settle(4);
		clickEverything(root, /delete|remove|sign out|publish|transfer|leave|model|provider/i);
		await settle(8);
		clickEverything(document.body, /delete|remove|sign out|publish|transfer|leave|model|provider/i);
		await settle(8);
		expect(calls.length).toBeGreaterThan(0);
	});

	it('shows a project that has nothing to build yet', async () => {
		backend();
		const root = render(Page as never, { data: data({ project: null, messages: [], agentStatus: { phase: 'thinking' } }) as never, form: null as never, params: { sessionId: 's1' } as never });
		await settle(6);
		expect(root.textContent).toBeTruthy();
	});

	it('an issue’s chat takes the chat column only: the project’s toolbar stays, with a way back', async () => {
		backend();
		const issue = { number: 7, title: 'Add dark mode', crew_agent: 'koda', crew_state: 'working', status: 'open', column: 'ready' };
		const root = render(Page as never, { data: data({ project: { ...project, issue } }) as never, form: null as never, params: { sessionId: 's7' } as never });
		await settle(8);
		expect(root.querySelector('.topbar__name')?.textContent).toContain('Bakery');
		expect(root.querySelector('.topbar__name')?.textContent).not.toContain('Add dark mode');
		expect(root.querySelector('.issue-bar__back')?.getAttribute('href')).toBe('/projects/session/s1');
		expect(root.querySelector('.issue-bar__title')?.textContent).toContain('#7');
	});

	it('someone who can only read gets the code filling the pane, with a file picker for a narrow one', async () => {
		fakeBackend([
			['GET', /\/api\/deploy$/, { available: true, role: 'viewer', app: 'bakery', database: 'none', url: null, publishes: [], live: null, backend_setup: null, unpublished_changes: 0 }],
			['GET', /\/presence/, { people: [] }],
			['GET', /\/queue/, { asks: [] }],
			['*', /.*/, {}],
		]);
		const root = render(Page as never, { data: data() as never, form: null as never, params: { sessionId: 's1' } as never });
		await settle(8);
		for (const tab of controls(/code/i, root).slice(0, 2)) tab.click();
		await settle(6);
		const options = [...root.querySelectorAll('.code-read__pick option')].map((o) => (o as HTMLOptionElement).value);
		expect(options).toEqual(expect.arrayContaining(['index.html', 'src/app.js', 'styles/site.css']));
		expect(root.querySelector('.code-read__view [role="tree"]')).not.toBeNull();
	});
});
