// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount, type Component } from 'svelte';
import SuggestionsTwin from './SuggestionsTwin.svelte';
import RunFailedTwin from './RunFailedTwin.svelte';
import ReminderTwin from './ReminderTwin.svelte';
import WithChatSend from './WithChatSend.svelte';
import ApprovalTwin from './ApprovalTwin.svelte';

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
