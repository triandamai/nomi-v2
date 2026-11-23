import { describe, expect, it } from 'vitest';
import type { RenderedMessage } from '$lib/adapters/api';
import { repeatsCard, threadEntries, turnDuration } from './thread';

let clock = 0;
function msg(partial: Partial<RenderedMessage> & { at?: string }): RenderedMessage {
	clock += 1;
	const { at, ...rest } = partial;
	return {
		id: `m${clock}`,
		sender: 'assistant',
		content: '',
		content_html: '',
		content_blocks: null,
		created_at: at ?? new Date(Date.UTC(2026, 9, 21, 7, 0, clock)).toISOString(),
		my_feedback: null,
		agent_display_name: 'Nomi',
		memory_count: 0,
		...rest,
	};
}

const options = { locale: 'en', now: new Date('2026-10-21T08:00:00Z'), timeZone: 'UTC', newFromId: null };

describe('threadEntries', () => {
	it('opens each day with its divider and folds thinking into the reply', () => {
		const ask = msg({ sender: 'user', content: 'Plan Bandung', at: '2026-10-20T10:00:00Z', agent_display_name: null });
		const old = msg({ content: 'Five steps.', at: '2026-10-20T10:00:09Z' });
		const user = msg({ sender: 'user', content: 'And food?', at: '2026-10-21T07:00:00Z', agent_display_name: null });
		const thinking = msg({ content: '🧠 x', content_blocks: [{ kind: 'reasoning', text: 'Looking at October.' }], at: '2026-10-21T07:00:02Z' });
		const reply = msg({ content: 'Rp 2,1 jt.', at: '2026-10-21T07:00:09Z' });
		const entries = threadEntries([ask, old, user, thinking, reply], options);
		expect(entries.map((entry) => entry.type)).toEqual(['day', 'user', 'reply', 'day', 'user', 'reply']);
		expect(entries[0]).toMatchObject({ label: 'Yesterday' });
		expect(entries[3]).toMatchObject({ label: 'Today' });
		expect(entries[5]).toMatchObject({ message: reply, reasoning: ['Looking at October.'], latest: true, turnStart: user.created_at });
		expect(entries[2]).toMatchObject({ latest: false });
		expect(entries[4]).toMatchObject({ latest: true });
		expect(entries[1]).toMatchObject({ latest: false });
	});

	it('marks where the person left off', () => {
		const user = msg({ sender: 'user', agent_display_name: null });
		const a = msg({ content: 'a' });
		const b = msg({ content: 'b' });
		const entries = threadEntries([user, a, b], { ...options, newFromId: a.id });
		expect(entries.map((entry) => entry.type)).toEqual(['day', 'user', 'new', 'reply', 'reply']);
		expect(entries[2]).toEqual({ type: 'new', count: 2 });
	});

	it('reads an error bubble as a failure, and a quota card too', () => {
		const user = msg({ sender: 'user', agent_display_name: null });
		const failed = msg({ content: 'DeepSeek is having trouble right now. Try again in a moment.', content_blocks: [{ kind: 'run_failed', retry: { target: 'last_message' } }] });
		const entries = threadEntries([user, failed], options);
		expect(entries.at(-1)).toMatchObject({ type: 'reply', failure: { kind: 'provider_down', retry: true } });
	});

	it('turns a follow-ups card into follow-ups for the reply before it, while they still apply', () => {
		const user = msg({ sender: 'user', agent_display_name: null });
		const reply = msg({ content: 'Done.' });
		const next = msg({ content: '', content_blocks: [{ kind: 'suggestions', items: ['Compare with September'] }] });
		const latest = threadEntries([user, reply, next], options);
		expect(latest.map((entry) => entry.type)).toEqual(['day', 'user', 'reply']);
		expect(latest.at(-1)).toMatchObject({ followUps: ['Compare with September'] });
		const later = threadEntries([user, reply, next, msg({ sender: 'user', agent_display_name: null })], options);
		expect(later.filter((entry) => entry.type === 'reply')[0]).toMatchObject({ followUps: [] });
	});
});

describe('turnDuration', () => {
	it('is how long the reply took after the person asked, in seconds', () => {
		expect(turnDuration('2026-10-21T07:00:00Z', '2026-10-21T07:00:09.400Z')).toBe('9s');
		expect(turnDuration('2026-10-21T07:00:00Z', '2026-10-21T07:02:05Z')).toBe('2m 5s');
		expect(turnDuration(null, '2026-10-21T07:02:05Z')).toBeUndefined();
		expect(turnDuration('2026-10-21T07:00:00Z', '2026-10-21T07:00:00.100Z')).toBe('0s');
	});
});

describe('action cards', () => {
	const decision = {
		kind: 'view',
		view: 'decision',
		question: 'Which direction should we take?',
		options: [
			{ label: 'Lean Operator', detail: 'Clean fintech', recommended: true },
			{ label: 'Studio Craft', detail: 'Warm editorial' },
			{ label: 'Dark Velocity', detail: 'High-contrast dark' },
		],
	};
	const card = (content: string) => msg({ agent_display_name: 'Sora', content, content_blocks: [decision] });

	it('draws a card under the text the same crew member says after it', () => {
		const user = msg({ sender: 'user', agent_display_name: null });
		const thinking = msg({ agent_display_name: 'Sora', content: '', content_blocks: [{ kind: 'reasoning', text: 'Three looks.' }] });
		const shown = card('');
		const reply = msg({ agent_display_name: 'Sora', content: 'Here are three looks for FlowBudget; I would take the first.' });
		const entries = threadEntries([user, thinking, shown, reply], options);
		expect(entries.map((entry) => entry.type)).toEqual(['day', 'user', 'reply']);
		expect(entries[2]).toMatchObject({ message: reply, cards: [shown], quiet: false, reasoning: ['Three looks.'] });
	});

	it('drops text that only repeats the options on the card', () => {
		const user = msg({ sender: 'user', agent_display_name: null });
		const shown = card('Directions saved as version 1: Lean Operator (lean-operator), Studio Craft (studio-craft), Dark Velocity (dark-velocity).');
		const reply = msg({ agent_display_name: 'Sora', content: '- **Lean Operator** (Recommended)\n- **Studio  craft**: warm\n- **Dark Velocity**: dark\n\nWhich one?' });
		const entries = threadEntries([user, shown, reply], options);
		expect(entries[2]).toMatchObject({ message: reply, cards: [shown], quiet: true });
	});

	it('keeps a card in place when no text follows it in the turn', () => {
		const user = msg({ sender: 'user', agent_display_name: null });
		const shown = card('');
		const other = msg({ agent_display_name: 'Nomi', content: 'Sora is on it.' });
		const next = msg({ sender: 'user', agent_display_name: null });
		const late = msg({ agent_display_name: 'Sora', content: 'Later.' });
		const entries = threadEntries([user, shown, other, next, late], options);
		expect(entries.filter((entry) => entry.type === 'reply').map((entry) => entry.type === 'reply' && entry.message.id)).toEqual([shown.id, other.id, late.id]);
		expect(entries[2]).toMatchObject({ message: shown, quiet: false, cards: [] });
	});

	it('only calls text a repeat when it names every option', () => {
		expect(repeatsCard('Lean Operator and Studio Craft', [card('')])).toBe(false);
		expect(repeatsCard('lean operator, studio craft or dark velocity?', [card('')])).toBe(true);
		expect(repeatsCard('Approve it?', [msg({ content_blocks: [{ kind: 'approval_request', id: 'a', status: 'pending' }] })])).toBe(false);
	});
});

describe('a to-do list reply', () => {
	it('draws only the card, not its fallback text (issue #335)', () => {
		const todo = msg({ agent_display_name: 'Koda', content: '📋 To-do list:\n[~] Configure package.json\n[ ] Set up Relay schema\n', content_blocks: [{ kind: 'todo_list', items: [{ id: '1', text: 'Configure package.json', status: 'in_progress' }] }] as never });
		const entries = threadEntries([msg({ sender: 'user', content: 'Build it' }), todo], options);
		const reply = entries.find((entry) => entry.type === 'reply');
		expect(reply?.type === 'reply' && reply.quiet).toBe(true);
	});
});

