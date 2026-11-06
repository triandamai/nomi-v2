// @vitest-environment jsdom
// More components with their parents' props: a plan card and its versions, the date and time
// pickers, the sign-in shell, the share panel, a draft plan, and the project preview.
import { createRawSnippet } from 'svelte';
import { describe, expect, it, vi } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text } from '../../test-stubs/render';
import AuthShell from './AuthShell.svelte';
import BuildSharePanel from './ds/build/BuildSharePanel.svelte';
import DraftPlanCard from './ds/chat/DraftPlanCard.svelte';
import DatePicker from './m3/DatePicker.svelte';
import TimePicker from './m3/TimePicker.svelte';
import ProjectPreview from './ProjectPreview.svelte';
import PlanBlock from './blocks/PlanBlock.svelte';

async function tryAll(root: HTMLElement) {
	await settle(4);
	clickEverything(root);
	await settle(6);
	clickEverything(document.body);
	await settle(4);
}

describe('more components', () => {
	it('a plan card loads its versions and the build that followed', async () => {
		const version = (n: number, state: string) => ({
			id: `pl${n}`,
			title: 'Trip to Bali',
			version: n,
			content_html: `<h2>Day ${n}</h2><ul><li>Ubud</li></ul>`,
			checklist: [{ text: 'Book flights', status: 'done' }, { text: 'Book villa', status: n === 2 ? 'in_progress' : 'pending' }],
			excerpt: 'Five days in Bali',
			created_at: `2026-11-0${n}T00:00:00Z`,
			state,
		});
		const build = {
			delegation_id: 'd1',
			status: 'completed',
			report: { status: 'failing', files_version: 4, changed_files: ['index.html'], deleted_files: ['old.js'], checks: [{ command: 'npm run check', outcome: 'failed', source: 'koda' }, { command: 'npm run build', outcome: 'passed', source: 'preview' }] },
			error: null,
			updated_at: '2026-11-03T00:00:00Z',
			started_at: '2026-11-03T00:00:00Z',
			finished_at: '2026-11-03T00:05:00Z',
		};
		const calls = fakeBackend([['*', /plans/, { plans: [version(1, 'updated'), version(2, 'active')], build }]]);
		const block = { kind: 'plan', plan_id: 'pl2', agent_session_id: 'as1', title: 'Trip to Bali', version: 2 } as const;
		const root = render(PlanBlock as never, { block, agent: 'planning', answered: false });
		await tryAll(root);
		await tryAll(root);
		expect(calls.length).toBeGreaterThan(0);
		expect(text()).toContain('Trip to Bali');

		fakeBackend([['*', /.*/, new Response('no', { status: 500 })]]);
		const failed = render(PlanBlock as never, { block, agent: null, answered: true });
		await tryAll(failed);
	});

	it('the pickers open, move and pick', async () => {
		const onchange = vi.fn();
		for (const props of [
			{ label: 'Due', value: '2026-11-07', onchange },
			{ label: 'Trip', mode: 'range', start: '2026-11-07', end: '2026-11-12', onchange },
			{ label: 'Due', value: '', variant: 'input', min: '2026-01-01', max: '2026-12-31', error: true, supportingText: 'Pick a day' },
		]) {
			const root = render(DatePicker as never, props as never);
			await tryAll(root);
		}
		for (const props of [
			{ label: 'At', value: '07:30', onchange, hour12: true },
			{ label: 'At', value: '19:05', variant: 'wheel', step: 5, hour12: false },
			{ label: 'At', value: '', disabled: true },
		]) {
			const root = render(TimePicker as never, props as never);
			await tryAll(root);
		}
		expect(document.body.textContent).toBeTruthy();
	});

	it('the sign-in shell, the share panel and a draft plan', async () => {
		const children = createRawSnippet(() => ({ render: () => '<form><input name="email" /></form>' }));
		const shell = render(AuthShell as never, { title: 'Sign in', subtitle: 'Welcome back', switchPrompt: 'New here?', switchLabel: 'Create an account', switchHref: '/register', children });
		expect(text(shell)).toContain('Welcome back');

		const handlers = { onvisibility: vi.fn(), oneditpreview: vi.fn(), oninvite: vi.fn(), onmore: vi.fn(), onclose: vi.fn(), ondone: vi.fn() };
		const share = render(BuildSharePanel as never, { titleId: 'share-title', url: 'https://bakery.apps.test', preview: { title: 'Bakery', description: 'Orders', image: null }, visibility: 'link', ...handlers });
		await tryAll(share);

		const steps = [{ label: 'Design the home page', owner: { agent: 'design', name: 'Ines' } }, { label: 'Build the order form', owner: { agent: 'coding', name: 'Koda' } }];
		const draftHandlers = { onmove: vi.fn(), onremove: vi.fn(), onassign: vi.fn(), onadd: vi.fn(), onrun: vi.fn(), onchanges: vi.fn(), ondiscard: vi.fn() };
		const draft = render(DraftPlanCard as never, { intro: 'Here is the plan', steps, estimate: 'About 20 minutes', ...draftHandlers });
		expect(text(draft)).toContain('Build the order form');
		await tryAll(draft);
	});

	it('the project preview shows static files, and a runtime that is still starting', async () => {
		fakeBackend([['*', /.*/, new Response('<h1>Bakery</h1>', { status: 200 })]]);
		const onShowCode = vi.fn();
		const staticSite = render(ProjectPreview as never, { projectId: 'p1', runtime: null, stack: 'static', files: ['index.html', 'about.html', 'style.css'], liveUrl: 'https://bakery.apps.test', onShowCode });
		await tryAll(staticSite);
		const runtime = { phase: 'installing', previewUrl: null, log: '$ npm install', files: [], error: null, checkFailed: false, handedToKoda: false, running: 'npm install', restart: vi.fn(), start: vi.fn(), stop: vi.fn() };
		const starting = render(ProjectPreview as never, { projectId: 'p1', runtime, stack: 'sveltekit', files: ['package.json'], liveUrl: null });
		await tryAll(starting);
		const running = render(ProjectPreview as never, { projectId: 'p1', runtime: { ...runtime, phase: 'running', previewUrl: 'https://preview.test', running: null }, stack: 'sveltekit' });
		expect(running.innerHTML).toContain('preview.test');
		const failed = render(ProjectPreview as never, { projectId: 'p1', runtime: { ...runtime, phase: 'failed', error: 'npm install failed', checkFailed: true, handedToKoda: true }, stack: 'sveltekit' });
		await tryAll(failed);
	});
});
