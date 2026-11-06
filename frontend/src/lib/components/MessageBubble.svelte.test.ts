// @vitest-environment jsdom
// A message with each kind of block the crew can send, and the person's own message with its
// edit, copy and feedback controls.
import { describe, expect, it, vi } from 'vitest';
import { clickEverything, fakeBackend, render, settle, text } from '../../test-stubs/render';
import MessageBubble from './MessageBubble.svelte';
import type { ContentBlock, RenderedMessage } from '$lib/types';

const reply = (blocks: ContentBlock[] | null, over: Partial<RenderedMessage> = {}): RenderedMessage => ({
	id: 'm2',
	sender: 'assistant',
	content: 'Here you go.',
	content_html: '<p>Here you go. <a href="https://kosku.id/a">1</a></p><pre data-lang="ts"><code>const a = 1</code></pre>',
	content_blocks: blocks,
	created_at: '2026-11-05T09:00:05Z',
	my_feedback: null,
	agent_display_name: 'Finley',
	memory_count: 2,
	steps: [{ tool: 'web_search', label: 'Searched the web', detail: 'kos bandung' } as never],
	sources: [{ title: 'Kos Mawar', url: 'https://kosku.id/a' } as never],
	used_memories: [{ id: 'mem1', content: 'Lives in Bandung', kind: 'fact' }],
	...over,
});

const blocks: ContentBlock[] = [
	{ kind: 'file_write', project_id: 'p1', path: 'index.html', content: '<h1>New</h1>', previous_content: '<h1>Old</h1>' },
	{ kind: 'file_writes', project_id: 'p1', files: [{ path: 'a.ts', content: 'a', previous_content: null }, { path: 'b.ts', content: 'b2', previous_content: 'b' }] },
	{ kind: 'file_delete', project_id: 'p1', path: 'old.ts', previous_content: 'x' },
	{ kind: 'todo_list', items: [{ content: 'Plan', status: 'completed' }, { content: 'Build', status: 'in_progress' }] as never },
	{ kind: 'table', variant: 'data', columns: [{ key: 'name', label: 'Name' }], rows: [{ name: 'Rice' }], title: 'Groceries', source: '1 row' },
	{ kind: 'workspace_connect', reason: 'not_connected', services: ['gmail'], agent_type: 'workspace' },
	{ kind: 'connector_connect', connector: 'hevy', reason: 'key_rejected', agent_type: 'health' },
	{ kind: 'project_link', project_id: 'p1', session_id: 's9', name: 'Bakery' },
	{ kind: 'quota_notice', reason: 'used_up' },
	{ kind: 'suggestions', items: ['Show last month', 'Set a budget'] },
	{ kind: 'run_failed', retry: { target: 'turn' } },
	{ kind: 'questions', intro: 'A few things first', questions: [{ question: 'Which city?', options: [{ label: 'Bandung' }, { label: 'Jakarta', detail: 'the capital' }] }, { question: 'Budget?', options: [{ label: 'Low' }], multiple: true }] },
	{ kind: 'approval_request', id: 'ap1', tool_name: 'gmail_send', description: 'Send an email to Ben', input: { to: 'ben@x.test' }, status: 'pending', decided_at: null },
	{ kind: 'reminder', reminder_id: 'r1', title: 'Pay rent', notes: 'Bu Sri', due_at: '2026-11-07T02:00:00Z', recurrence: 'monthly', state: 'open' },
	{ kind: 'member_joined', name: 'Ben', role: 'build', publisher: 'Ana' },
	{ kind: 'project_event', event: 'deploy', ok: true, text: 'Published version 4' },
	{ kind: 'stopped', tasks: [{ agent: 'money', title: 'Summing up', detail: null, state: 'stopped' }, { agent: 'coding', title: 'Building', state: 'waiting' }] },
	{ kind: 'voice', url: '/voice/1.mp3', text: 'Halo', voice: 'Rachel' },
	{ kind: 'view', type: 'list', items: [{ title: 'Kos Mawar', subtitle: 'Rp 1.5 juta' }] } as never,
];

const props = (message: RenderedMessage, extra: Record<string, unknown> = {}) => ({
	message,
	onregenerate: vi.fn(),
	ondelete: vi.fn(),
	onrerun: vi.fn(),
	...extra,
});

describe('MessageBubble', () => {
	it('shows a reply with every kind of block', async () => {
		fakeBackend([['*', /.*/, { type: 'success', status: 200, data: { success: true } }]]);
		const root = render(MessageBubble as never, props(reply(blocks), { reasoning: ['First I looked at the numbers'], savedMemories: [{ id: 'mem2', content: 'Takes the MRT', kind: 'routine' }], thread: 'start' }) as never);
		await settle(6);
		const shown = text(root);
		for (const piece of ['Kos Mawar', 'Groceries', 'Connect Gmail', 'a.ts']) expect(shown).toContain(piece);
		clickEverything(root, /delete|remove/i);
		await settle(6);
		clickEverything(document.body, /delete|remove/i);
		await settle(4);
	});

	it('shows the person’s own message with its controls', async () => {
		fakeBackend([['*', /.*/, { type: 'success', status: 200, data: { success: true } }]]);
		const mine = reply(null, { id: 'm1', sender: 'user', content: 'How much?', content_html: '<p>How much?</p>', agent_display_name: null, edited_at: '2026-11-05T09:01:00Z', versions: 2, my_feedback: 'up', steps: [], sources: [] });
		const root = render(MessageBubble as never, props(mine, { chained: true, first: true }) as never);
		expect(text(root)).toContain('How much?');
		clickEverything(root, /delete|remove/i);
		await settle(6);
	});

	it('shows thinking only, a deleted reply, and a hand-off', () => {
		render(MessageBubble as never, props(reply(null), { thinkingOnly: true, reasoning: ['Hmm'] }) as never);
		render(MessageBubble as never, props(reply(null, { deleted: true }), {}) as never);
		const handed = render(MessageBubble as never, props(reply([{ kind: 'plan', plan_id: 'pl1', agent_session_id: 'as1', title: 'Trip plan', version: 2 }]), { handoffFrom: 'Rena', thread: 'end', answered: true }) as never);
		expect(handed.textContent).toBeTruthy();
	});
});
