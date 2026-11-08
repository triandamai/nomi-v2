// @vitest-environment jsdom
// The issue sheet (properties, "Work immediately", the Markdown description), the crew picker,
// and the project chat's strip of what the crew is doing.
import { afterEach, describe, expect, it, vi } from 'vitest';
import { fakeBackend, render, settle, text, type } from '../../../test-stubs/render';
import { localToday } from '$lib/issues';
import IssueEditorSheet from './IssueEditorSheet.svelte';
import IssueWorkStrip from './IssueWorkStrip.svelte';
import IssueFollowUp from './IssueFollowUp.svelte';
import IssueBoard from './IssueBoard.svelte';
import MarkdownEditor from '../MarkdownEditor.svelte';

const issue = (over: Record<string, unknown> = {}) => ({
	number: 1, title: 'Checkout button broken', body: 'It **500s**', kind: 'bug', severity: 'high', status: 'open', created_by: 'Ana', mine: true,
	closed_at: null, created_at: '2026-11-01T00:00:00Z', updated_at: '2026-11-01T00:00:00Z', column_id: 'c-ready', position: 1,
	start_on: null, due_on: null, points: 3, crew_agent: 'coding', assigned_by_agent: null, crew_state: null, session_id: null, ...over,
});

const labels = { kindLabel: (k: string) => k, severityLabel: (s: string) => s };

function submit(root: Element) {
	root.querySelector('form')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
}

afterEach(() => {
	document.body.innerHTML = '';
});

describe('issue sheet', () => {
	it('"Work immediately" starts it today and files it straight to Ready to dev', async () => {
		const calls = fakeBackend([['POST', /\/issues$/, issue()]]);
		const onsaved = vi.fn();
		const root = render(IssueEditorSheet as never, { open: true, projectId: 'p1', issue: null, onsaved, ...labels });
		await settle(4);
		type(root.querySelector('#issue-title'), 'Add dark mode');
		[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Work immediately')!.click();
		await settle(2);
		expect(text(root)).toContain('Koda picks it up');
		expect(text(root)).toContain('Create and start');
		submit(root);
		await settle(6);
		const sent = calls.find((c) => c.method === 'POST')!.body as Record<string, unknown>;
		expect(sent).toMatchObject({ title: 'Add dark mode', start_now: true, start_on: localToday(), crew_agent: 'coding', kind: 'task' });
		expect(onsaved).toHaveBeenCalled();
	});

	it('editing with "Work immediately" saves, then moves it to the end of Ready to dev', async () => {
		const board = { columns: [{ id: 'c-back', name: 'Backlog', position: 0, fixed: 'backlog', issues: [issue({ column_id: 'c-back' })] }, { id: 'c-ready', name: 'Ready to dev', position: 1, fixed: 'ready', issues: [issue({ number: 2 })] }] };
		const calls = fakeBackend([
			['PATCH', /\/issues\/1$/, issue()],
			['GET', /\/board$/, board],
			['POST', /\/issues\/1\/move$/, null],
		]);
		const root = render(IssueEditorSheet as never, { open: true, projectId: 'p1', issue: issue({ column_id: 'c-back' }), onsaved: vi.fn(), ...labels });
		await settle(4);
		expect((root.querySelector('#issue-title') as HTMLTextAreaElement).value).toBe('Checkout button broken');
		[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Work immediately')!.click();
		await settle(2);
		submit(root);
		await settle(8);
		expect(calls.find((c) => c.path.endsWith('/issues/1/move'))?.body).toEqual({ column_id: 'c-ready', index: 1 });
	});

	it('refuses a deadline before the start date', async () => {
		const calls = fakeBackend([]);
		const root = render(IssueEditorSheet as never, { open: true, projectId: 'p1', issue: issue({ start_on: '2026-11-10', due_on: '2026-11-01' }), onsaved: vi.fn(), ...labels });
		await settle(4);
		submit(root);
		await settle(2);
		expect(text(root)).toContain('The deadline can’t be before the start date.'.replace('’', "'"));
		expect(calls.length).toBe(0);
	});

	it('the crew picker finds crew by name or by what they do', async () => {
		fakeBackend([]);
		const root = render(IssueEditorSheet as never, { open: true, projectId: 'p1', issue: null, onsaved: vi.fn(), ...labels });
		await settle(4);
		const input = root.querySelector<HTMLInputElement>('[role="combobox"]')!;
		input.dispatchEvent(new FocusEvent('focus'));
		type(input, 'budg');
		await settle(2);
		const options = [...root.querySelectorAll('[role="option"]')].map((o) => o.textContent);
		expect(options).toHaveLength(1);
		expect(options[0]).toContain('Dana');
		input.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', bubbles: true }));
		await settle(2);
		expect(input.placeholder).toBe('Dana');
	});
});

describe('markdown editor', () => {
	it('shows Markdown formatted, read-only', async () => {
		const root = render(MarkdownEditor as never, { value: '# Steps\n\n- open **cart**\n- [ ] pay', label: 'Description', editable: false });
		await settle(2);
		expect(root.querySelector('h1')?.textContent).toBe('Steps');
		expect(root.querySelector('strong')?.textContent).toBe('cart');
		expect(root.querySelector('[data-type="taskList"]')).not.toBeNull();
	});
});

describe('crew work in the project chat', () => {
	it('says who is working on which issue and opens its chat', async () => {
		const board = {
			columns: [
				{ id: 'c-back', name: 'Backlog', position: 0, fixed: 'backlog', issues: [issue({ number: 4, crew_state: null })] },
				{ id: 'c-ready', name: 'Ready to dev', position: 1, fixed: 'ready', issues: [
					issue({ number: 1, crew_state: 'working', session_id: 's-1' }),
					issue({ number: 2, crew_state: 'queued', crew_agent: 'money' }),
					issue({ number: 3, crew_state: 'failed', session_id: 's-3' }),
					issue({ number: 5, crew_state: 'done', session_id: 's-5' }),
				] },
			],
		};
		const calls = fakeBackend([['GET', /\/board$/, board]]);
		const listeners: ((raw: string) => void)[] = [];
		const onmore = vi.fn();
		const root = render(IssueWorkStrip as never, { projectId: 'p1', onmore, subscribe: (l: (raw: string) => void) => (listeners.push(l), () => {}) });
		await settle(6);
		const shown = text(root).replace(/’/g, "'");
		expect(shown).toContain('Koda is working on issue #1');
		expect(shown).toContain('Dana will work on issue #2');
		expect(shown).toContain("Koda couldn't finish issue #3");
		expect(shown).not.toContain('#4');
		expect(root.querySelector('a')?.getAttribute('href')).toBe('/projects/session/s-1');
		[...root.querySelectorAll('button')].find((b) => b.textContent?.includes('1 more'))!.click();
		expect(onmore).toHaveBeenCalled();
		for (const listener of listeners) listener('{"kind":"DeployChanged","event":"issues.changed"}');
		await settle(4);
		expect(calls.filter((c) => c.path.endsWith('/board')).length).toBe(2);
	});
});

describe('finishing an issue', () => {
	it('asks whether it is done, and "Not yet" leaves the composer to say what is missing', async () => {
		const ondone = vi.fn(async () => {});
		const root = render(IssueFollowUp as never, { number: 4, crewAgent: 'coding', ondone });
		await settle(2);
		expect(text(root)).toContain('Koda finished. Is issue #4 done?');
		[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Yes, mark as done')!.click();
		await settle(2);
		expect(ondone).toHaveBeenCalled();
		[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Not yet')!.click();
		await settle(2);
		expect(text(root).replace(/’/g, "'")).toContain("Tell Koda what's still missing");
	});

	it('says so when marking it done fails', async () => {
		const root = render(IssueFollowUp as never, { number: 4, crewAgent: 'coding', ondone: async () => { throw new Error('no'); } });
		await settle(2);
		[...root.querySelectorAll('button')].find((b) => b.textContent?.trim() === 'Yes, mark as done')!.click();
		await settle(4);
		expect(text(root).replace(/’/g, "'")).toContain("Couldn't mark it done");
	});

	it('the board ends with Done and Closed, fixed, holding the closed issues', async () => {
		fakeBackend([]);
		const board = { columns: [
			{ id: 'c-back', name: 'Backlog', position: 0, fixed: 'backlog', issues: [] },
			{ id: 'c-ready', name: 'Ready to dev', position: 1, fixed: 'ready', issues: [issue({ number: 1, crew_state: 'done', session_id: 's-1' })] },
			{ id: 'c-done', name: 'Done', position: 1000, fixed: 'done', issues: [issue({ number: 2, status: 'closed', crew_state: 'done', column_id: 'c-done' })] },
			{ id: 'c-closed', name: 'Closed', position: 1001, fixed: 'closed', issues: [issue({ number: 3, status: 'closed', crew_state: 'failed', column_id: 'c-closed' })] },
		] };
		const root = render(IssueBoard as never, { projectId: 'p1', board, canEditColumns: true, onopen: vi.fn(), onchange: vi.fn(), ...labels });
		await settle(2);
		const shown = text(root);
		for (const piece of ['Koda finished: check it', 'Done by Koda', 'Checked and finished']) expect(shown).toContain(piece);
		expect([...root.querySelectorAll('.column__name')].map((h) => h.firstChild?.textContent?.trim())).toEqual(['Backlog', 'Ready to dev', 'Done', 'Closed']);
		// Fixed columns have no menu, and a closed one that failed isn't offered "Try again".
		expect(root.querySelectorAll('[aria-label^="Edit column"]').length).toBe(0);
		expect(shown).not.toContain('Try again');
	});
});
