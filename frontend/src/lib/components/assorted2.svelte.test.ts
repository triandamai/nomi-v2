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

async function tryAll(root: HTMLElement) {
	await settle(4);
	clickEverything(root);
	await settle(6);
	clickEverything(document.body);
	await settle(4);
}

describe('more components', () => {
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

	it('the sandbox preview states: waking with seconds, failed with log tail, retry and browser fallback', async () => {
		{
			fakeBackend([['*', /.*/, new Response('{}', { status: 200 })]]);
			const base = { phase: 'waking', previewUrl: null, log: '', files: [], error: null, checkFailed: false, handedToKoda: false, running: null, restart: vi.fn(), retry: vi.fn(), start: vi.fn(), stop: vi.fn() };
			const onUseBrowser = vi.fn();
			const waking = render(ProjectPreview as never, { projectId: 'p1', runtime: base, stack: 'sveltekit', onUseBrowser });
			await settle(2);
			await new Promise((resolve) => setTimeout(resolve, 1200));
			await settle(2);
			expect(text(waking)).toContain('Waking the sandbox');
			expect(text(waking)).toMatch(/[1-9]\s?s\b/);
			const fallback = [...waking.querySelectorAll('button')].find((b) => b.textContent?.includes('Run in my browser instead'));
			expect(fallback).toBeTruthy();
			fallback!.click();
			expect(onUseBrowser).toHaveBeenCalledTimes(1);

			const starting = render(ProjectPreview as never, { projectId: 'p1', runtime: { ...base, phase: 'starting' }, stack: 'sveltekit', onUseBrowser });
			expect(text(starting)).toContain('Starting the app');

			const failedRuntime = { ...base, phase: 'failed', error: 'sandbox crashed', log: 'line one\nnpm ERR! last line' };
			const failed = render(ProjectPreview as never, { projectId: 'p1', runtime: failedRuntime, stack: 'sveltekit', onUseBrowser });
			await settle(2);
			expect(text(failed)).toContain('sandbox crashed');
			expect(text(failed)).toContain('npm ERR! last line');
			const retry = [...failed.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Retry');
			expect(retry).toBeTruthy();
			retry!.click();
			expect(failedRuntime.retry).toHaveBeenCalledTimes(1);
			expect(failedRuntime.restart).not.toHaveBeenCalled();
			const failedFallback = [...failed.querySelectorAll('button')].find((b) => b.textContent?.includes('Run in my browser instead'));
			failedFallback!.click();
			expect(onUseBrowser).toHaveBeenCalledTimes(2);

			// The browser runtime has neither: no Retry-to-sandbox, no fallback button.
			const { retry: _r, ...browser } = failedRuntime;
			const plain = render(ProjectPreview as never, { projectId: 'p1', runtime: browser, stack: 'sveltekit' });
			expect(text(plain)).not.toContain('Run in my browser instead');
			expect([...plain.querySelectorAll('button')].some((b) => b.textContent?.trim() === 'Retry')).toBe(false);
		}
	});
});
