import { describe, expect, it } from 'vitest';
import { afterRerun } from './rerun';
import type { RenderedMessage } from './types';

const message = (id: string, sender: 'user' | 'assistant', extra: Partial<RenderedMessage> = {}): RenderedMessage => ({
	id,
	sender,
	content: id,
	content_html: `<p>${id}</p>`,
	content_blocks: null,
	created_at: '2026-10-31T00:00:00Z',
	my_feedback: null,
	agent_display_name: null,
	memory_count: 0,
	...extra,
});

describe('afterRerun', () => {
	const reminder = message('r', 'assistant', { content_blocks: [{ kind: 'reminder', reminder_id: 'x', title: 'Rent', due_at: '2026-11-01T00:00:00Z' } as never] });

	it("drops the turn's replies and updates the person's message", () => {
		const thread = [message('a', 'user'), message('b', 'assistant'), message('q', 'user'), message('c', 'assistant'), reminder, message('d', 'assistant')];
		const next = afterRerun(thread, 'q', { content: 'q2', edited_at: 'now', versions: 1 });
		expect(next.map((m) => m.id)).toEqual(['a', 'b', 'q', 'r']);
		expect(next[2]).toMatchObject({ content: 'q2', edited_at: 'now', versions: 1 });
	});

	it('leaves a thread without that message alone', () => {
		const thread = [message('a', 'user')];
		expect(afterRerun(thread, 'zz', {})).toBe(thread);
	});
});
