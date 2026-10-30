// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount, type Component } from 'svelte';
import SuggestionsTwin from './SuggestionsTwin.svelte';
import RunFailedTwin from './RunFailedTwin.svelte';
import ReminderTwin from './ReminderTwin.svelte';
import WithChatSend from './WithChatSend.svelte';
import ApprovalTwin from './ApprovalTwin.svelte';
import QuestionsTwin from './QuestionsTwin.svelte';
import ConnectTwin from './ConnectTwin.svelte';
import TableTwin from './TableTwin.svelte';

// The chat's blocks drawn with the design-system twins keep what the old blocks did: the same
// requests, the same results.

let mounted: ReturnType<typeof mount>[] = [];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function render(component: Component<any>, props: Record<string, unknown>): HTMLElement {
	const target = document.body.appendChild(document.createElement('div'));
	mounted.push(mount(component, { target, props }));
	flushSync();
	return target;
}

const settle = async () => {
	await new Promise((resolve) => setTimeout(resolve));
	flushSync();
	await tick();
};

const buttonNamed = (root: ParentNode, name: string) => [...root.querySelectorAll('button')].find((button) => button.textContent?.trim() === name)!;

afterEach(() => {
	for (const component of mounted) unmount(component);
	mounted = [];
	document.body.innerHTML = '';
	vi.unstubAllGlobals();
});

describe('SuggestionsTwin', () => {
	const block = { kind: 'suggestions', items: ['Show last month', 'Set a budget'] };

	it('sends the picked suggestion as the next message', () => {
		const send = vi.fn();
		const root = render(WithChatSend, { send, block: SuggestionsTwin, props: { block } });
		buttonNamed(root, 'Set a budget').click();
		expect(send).toHaveBeenCalledWith('Set a budget');
	});

	it('is gone once the person has said something else', () => {
		const root = render(WithChatSend, { send: vi.fn(), block: SuggestionsTwin, props: { block, answered: true } });
		expect(root.querySelector('button')).toBeNull();
	});
});

describe('RunFailedTwin', () => {
	it('says what went wrong and tries the run again, taking the message away', async () => {
		const fetchMock = vi.fn(async () => new Response(JSON.stringify({ type: 'success' })));
		vi.stubGlobal('fetch', fetchMock);
		const onremove = vi.fn();
		const root = render(RunFailedTwin, { block: { kind: 'run_failed', retry: { target: 'turn' } }, messageId: 'm1', text: 'The model timed out.', onremove });
		expect(root.textContent).toContain('The model timed out.');
		buttonNamed(root, 'Try again').click();
		await settle();
		expect(fetchMock).toHaveBeenCalledWith('?/retryRun', expect.objectContaining({ method: 'POST' }));
		expect(onremove).toHaveBeenCalledOnce();
	});

	it('offers nothing to retry when there is nothing to rerun, and says why a retry failed', async () => {
		expect(render(RunFailedTwin, { block: { kind: 'run_failed' }, messageId: 'm1' }).querySelector('button')).toBeNull();

		vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({ type: 'failure', data: { error: 'Already running' } }))));
		const root = render(RunFailedTwin, { block: { kind: 'run_failed', retry: { target: 'turn' } }, messageId: 'm2' });
		buttonNamed(root, 'Try again').click();
		await settle();
		expect([...root.querySelectorAll('[role="alert"]')].at(-1)?.textContent).toBe('Already running');
	});
});

describe('ReminderTwin', () => {
	const block = { kind: 'reminder', reminder_id: 'r1', title: 'Pay rent', due_at: '2026-10-30T09:00:00Z' };

	it('marks the reminder done', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);
		const root = render(ReminderTwin, { block });
		buttonNamed(root, 'Mark done').click();
		await settle();
		expect(fetchMock).toHaveBeenCalledWith('/reminders/r1', expect.objectContaining({ body: JSON.stringify({ action: 'done' }) }));
		expect(root.querySelector('[role="status"]')).not.toBeNull();
		expect(buttonNamed(root, 'Mark done')).toBeUndefined();
	});

	it('snoozes it for an hour', async () => {
		const fetchMock = vi.fn(async () => new Response(null, { status: 204 }));
		vi.stubGlobal('fetch', fetchMock);
		const root = render(ReminderTwin, { block });
		buttonNamed(root, 'Snooze 1h').click();
		await settle();
		expect(fetchMock).toHaveBeenCalledWith('/reminders/r1', expect.objectContaining({ body: JSON.stringify({ action: 'snooze', minutes: 60 }) }));
	});

	it('shows a reminder already handled elsewhere as gone, not as two dead buttons', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response(null, { status: 404 })));
		const root = render(ReminderTwin, { block });
		buttonNamed(root, 'Mark done').click();
		await settle();
		expect(buttonNamed(root, 'Mark done')).toBeUndefined();
	});

	it('starts settled when it was answered before the chat loaded', () => {
		const root = render(ReminderTwin, { block: { ...block, state: 'done' } });
		expect(buttonNamed(root, 'Mark done')).toBeUndefined();
		expect(root.querySelector('[role="status"]')).not.toBeNull();
	});
});

describe('ApprovalTwin', () => {
	const block = { kind: 'approval_request', id: 'a1', tool_name: 'send_email', description: 'Send the invoice to Bu Sri', input: {}, status: 'pending', decided_at: null };

	it('asks for the OK on the tool, and approves for good when asked to', async () => {
		const fetchMock = vi.fn(async () => new Response(JSON.stringify({ type: 'success' })));
		vi.stubGlobal('fetch', fetchMock);
		const root = render(ApprovalTwin, { block, messageId: 'm1', agent: 'coding' });
		expect(root.textContent).toContain('send_email');
		root.querySelector<HTMLInputElement>('input[type="checkbox"]')!.click();
		buttonNamed(root, 'Approve').click();
		await settle();
		const body = (fetchMock.mock.calls[0] as unknown as [string, RequestInit])[1].body as FormData;
		expect([body.get('messageId'), body.get('decision'), body.get('remember')]).toEqual(['m1', 'approve', 'true']);
	});

	it('denies', async () => {
		const fetchMock = vi.fn(async () => new Response(JSON.stringify({ type: 'success' })));
		vi.stubGlobal('fetch', fetchMock);
		const root = render(ApprovalTwin, { block, messageId: 'm1' });
		buttonNamed(root, 'Deny').click();
		await settle();
		expect(((fetchMock.mock.calls[0] as unknown as [string, RequestInit])[1].body as FormData).get('decision')).toBe('deny');
	});

	it('says what to do when the turn already moved on, and offers no buttons', async () => {
		vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify({ type: 'failure', status: 409, data: {} }))));
		const root = render(ApprovalTwin, { block, messageId: 'm1' });
		buttonNamed(root, 'Approve').click();
		await settle();
		expect(buttonNamed(root, 'Approve')).toBeUndefined();
		expect([...root.querySelectorAll('[role="alert"]')].at(-1)?.textContent).toBeTruthy();
	});

	it('shows what was decided, without buttons', () => {
		const root = render(ApprovalTwin, { block: { ...block, status: 'approved' }, messageId: 'm1' });
		expect(buttonNamed(root, 'Approve')).toBeUndefined();
		expect(root.textContent).toContain('Approved');
	});
});

describe('QuestionsTwin', () => {
	const block = {
		kind: 'questions',
		intro: 'A few things first',
		questions: [
			{ question: 'Where to?', options: [{ label: 'Bali' }, { label: 'Lombok', detail: 'quieter' }] },
			{ question: 'What for?', multiple: true, options: [{ label: 'Beach' }, { label: 'Food' }] },
		],
	};

	it('asks one question at a time and sends the answers as the next message', async () => {
		const send = vi.fn();
		const root = render(WithChatSend, { send, block: QuestionsTwin, props: { block, agent: 'planning' } });
		expect(root.textContent).toContain('Where to?');
		root.querySelectorAll<HTMLInputElement>('input[type="radio"]')[1].click();
		flushSync();
		buttonNamed(root, 'Next').click();
		flushSync();
		await tick();
		expect(root.textContent).toContain('What for?');
		root.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')[1].click();
		flushSync();
		buttonNamed(root, 'Review answers').click();
		flushSync();
		expect(send).toHaveBeenCalledWith('Where to? Lombok\nWhat for? Food');
		expect(root.textContent).toContain('Answered 2 questions');
	});

	it('skips them all with Escape, and says so in the chat', () => {
		const send = vi.fn();
		const root = render(WithChatSend, { send, block: QuestionsTwin, props: { block } });
		root.querySelector('section, [role="group"], div')!.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', bubbles: true }));
		flushSync();
		expect(send).toHaveBeenCalledWith('Skip these questions and go with your best guess.');
	});

	it('folds to one line once the person replied', () => {
		const root = render(WithChatSend, { send: vi.fn(), block: QuestionsTwin, props: { block, answered: true } });
		expect(root.querySelector('input')).toBeNull();
		expect(root.textContent).toContain('Answered 2 questions');
	});
});

describe('ConnectTwin', () => {
	it('says who needs which Google service and what it can read, and links to connecting it', () => {
		const root = render(ConnectTwin, { block: { kind: 'workspace_connect', reason: 'not_connected', services: ['gmail'], agent_type: 'workspace' } });
		expect(root.textContent).toContain('Tara needs Gmail to go on with this.');
		expect(root.textContent).toContain('Read, search and draft replies');
		expect(root.querySelector('a')!.getAttribute('href')).toContain('services=gmail');
	});

	it('asks for a new key when the old one stopped working', () => {
		const root = render(ConnectTwin, { block: { kind: 'connector_connect', connector: 'hevy', reason: 'key_rejected', agent_type: 'health' } });
		expect(root.textContent).toContain('Hevy stopped accepting the saved key. Raga needs a new one to go on.');
		expect(root.querySelector('a')!.getAttribute('href')).toBe('/connections/hevy?reason=key_rejected');
	});

	it('knows who asked on older cards from the connector itself', () => {
		const root = render(ConnectTwin, { block: { kind: 'connector_connect', connector: 'hevy', reason: 'not_connected' } });
		expect(root.textContent).toContain('Raga needs Hevy');
	});

	it('goes away on Not now', () => {
		const root = render(ConnectTwin, { block: { kind: 'connector_connect', connector: 'hevy', reason: 'not_connected' } });
		buttonNamed(root, 'Not now').click();
		flushSync();
		expect(root.textContent?.trim()).toBe('');
	});
});

describe('TableTwin', () => {
	const columns = [
		{ key: 'email', label: 'Email' },
		{ key: 'visits', label: 'Visits' },
	];
	const rows = [
		{ email: 'a@b.c', visits: 3 },
		{ email: 'd@e.f', visits: null },
	];

	it('names the table and says where its rows came from', () => {
		const root = render(TableTwin, { block: { kind: 'table', variant: 'data', columns, rows, title: 'Query results', source: '2 rows' } });
		expect(root.textContent).toContain('Query results');
		expect(root.textContent).toContain('2 rows');
		expect([...root.querySelectorAll('th')].map((th) => th.textContent?.trim())).toEqual(['Email', 'Visits']);
		expect(root.querySelector('tbody tr td')?.textContent?.trim()).toBe('a@b.c');
	});

	it('keeps the old table for one from before tables had a title', () => {
		const root = render(TableTwin, { block: { kind: 'table', variant: 'data', columns, rows } });
		expect(root.textContent).not.toContain('Query results');
		expect(root.textContent).toContain('a@b.c');
	});
});
