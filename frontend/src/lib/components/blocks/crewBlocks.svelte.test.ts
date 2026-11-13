// @vitest-environment jsdom
import { afterEach, describe, expect, it, vi } from 'vitest';
import { flushSync, mount, tick, unmount, type Component } from 'svelte';
import WithChatSend from './twins/WithChatSend.svelte';
import StoppedBlock from './StoppedBlock.svelte';
import ProjectEventBlock from './ProjectEventBlock.svelte';
import SavedMemoriesTwin from './twins/SavedMemoriesTwin.svelte';
import ViewBlock from '../views/ViewBlock.svelte';

// The crew's own cards drawn with the design system: what a stop stopped, what was remembered, a
// project's events, a boundary, quick replies and a background task.

let mounted: ReturnType<typeof mount>[] = [];

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function render(component: Component<any>, props: Record<string, unknown>): HTMLElement {
	const target = document.body.appendChild(document.createElement('div'));
	mounted.push(mount(component, { target, props }));
	flushSync();
	return target;
}
const button = (root: ParentNode, text: string) => [...root.querySelectorAll('button')].find((b) => b.textContent?.includes(text))!;

afterEach(() => {
	for (const component of mounted) unmount(component);
	mounted = [];
	document.body.innerHTML = '';
	vi.unstubAllGlobals();
});

describe('StoppedBlock', () => {
	const block = {
		kind: 'stopped',
		tasks: [
			{ agent: 'money', title: 'Reply in progress', state: 'stopped' },
			{ agent: 'chitchat', title: 'Message waiting its turn', state: 'waiting' },
		],
	};

	it('says what it stopped, task by task, and resumes through the chat', () => {
		const send = vi.fn();
		const root = render(WithChatSend, { send, block: StoppedBlock, props: { block, text: 'Stopped Money.' } });
		expect(root.textContent).toContain('Stopped Money.');
		expect(root.textContent).toContain('Supervisor stopped 2 tasks');
		expect(root.textContent).toContain('Message waiting its turn');
		button(root, 'Resume').click();
		expect(send).toHaveBeenCalledWith('Pick up where you left off');
	});

	it('offers no resume once the person has moved on', () => {
		const root = render(WithChatSend, { send: vi.fn(), block: StoppedBlock, props: { block, answered: true } });
		expect(button(root, 'Resume')).toBeUndefined();
	});
});

describe('SavedMemoriesTwin', () => {
	it('shows what was remembered and forgets it on undo', async () => {
		const fetchMock = vi.fn(async () => new Response(JSON.stringify({ type: 'success', status: 200, data: '[{"forgotten":1},"m1"]' })));
		vi.stubGlobal('fetch', fetchMock);
		const root = render(SavedMemoriesTwin, { memories: [{ id: 'm1', content: 'Vegetarian', kind: 'preference' }] });
		expect(root.textContent).toContain('Saved to memory: Vegetarian');
		button(root, 'Undo').click();
		await new Promise((resolve) => setTimeout(resolve));
		flushSync();
		await tick();
		expect(fetchMock).toHaveBeenCalledWith('/memory?/forget', expect.objectContaining({ method: 'POST' }));
		expect(root.textContent).not.toContain('Vegetarian');
	});
});

describe('ProjectEventBlock', () => {
	it('shows a publish that went live as the live card, and a failed one as a logged line', () => {
		const root = render(ProjectEventBlock, {
			block: { kind: 'project_event', event: 'deploy', ok: true, text: 'production · published · shop.example.app' },
			time: '15:38',
		});
		expect(root.querySelector('a[href="https://shop.example.app"]')).not.toBeNull();
		expect(root.textContent).toContain('Published to production');
		expect(root.textContent).toContain('15:38');
		const failed = render(ProjectEventBlock as never, { block: { kind: 'project_event', event: 'deploy', ok: false, text: 'production · failed · build_image' }, time: '15:40' } as never);
		expect(failed.textContent).toContain('production · failed · build_image');
	});
});

describe('views', () => {
	const show = (block: unknown, send = vi.fn(), extra: Record<string, unknown> = {}) =>
		render(WithChatSend, { send, block: ViewBlock, props: { block, ...extra } });

	it('a boundary offers what the crew can do instead, sent as the next message', () => {
		const send = vi.fn();
		const root = show(
			{
				view: 'boundary',
				title: 'I can’t move money',
				reason: 'Nomi only records.',
				alternatives: [{ title: 'Log it once it’s done', agent: 'money', message: 'Log the transfer' }],
			},
			send,
		);
		expect(root.textContent).toContain('Nomi only records.');
		button(root, 'Log it once it’s done').click();
		expect(send).toHaveBeenCalledWith('Log the transfer');
	});

	it('quick replies answer in one tap, then go', () => {
		const send = vi.fn();
		const root = show({ view: 'quick_replies', replies: ['This week', 'This month'] }, send);
		button(root, 'This month').click();
		flushSync();
		expect(send).toHaveBeenCalledWith('This month');
		expect(button(root, 'This week')).toBeUndefined();
	});

	it('a background task card says it runs without you, with its command, and settles when it ends', () => {
		const running = show({ view: 'task', id: 't1', title: 'audit-export', status: 'running', detail: 'npm run export', agent: 'coding' });
		expect(running.textContent).toContain('npm run export');
		expect(running.textContent).toContain('in Inbox');
		const cancelled = show({ view: 'task', id: 't1', title: 'audit-export', status: 'cancelled', detail: 'npm run export' });
		expect(cancelled.textContent).toContain('Cancelled');
		expect(cancelled.textContent).not.toContain('in Inbox');
	});
});
