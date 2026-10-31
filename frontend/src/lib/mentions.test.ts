import { describe, expect, it } from 'vitest';
import { filterMentions, insertMention, mentionAt, MAX_MENTION_OPTIONS } from './mentions';
import type { MentionOption } from '$lib/components/ds/chat/mention';

const crew: MentionOption[] = [
	{ kind: 'crew', agent: 'dana', name: 'Dana', role: 'Money' },
	{ kind: 'crew', agent: 'kala', name: 'Kala', role: 'Reminders' },
	{ kind: 'crew', agent: 'koda', name: 'Koda', role: 'Code' },
];

describe('mentionAt', () => {
	it('finds the "@query" right before the caret', () => {
		expect(mentionAt('@da', 3)).toEqual({ start: 0, query: 'da' });
		expect(mentionAt('ask @Ko', 7)).toEqual({ start: 4, query: 'Ko' });
		expect(mentionAt('hi @', 4)).toEqual({ start: 3, query: '' });
	});

	it('ignores an email address and a finished mention', () => {
		expect(mentionAt('dana@ex', 7)).toBeNull();
		expect(mentionAt('@Dana coffee', 12)).toBeNull();
	});
});

describe('filterMentions', () => {
	it('puts names starting with the query first', () => {
		expect(filterMentions(crew, 'k').map((o) => o.name)).toEqual(['Kala', 'Koda']);
		expect(filterMentions(crew, 'a').map((o) => o.name)).toEqual(['Dana', 'Kala', 'Koda']);
		expect(filterMentions(crew, 'zz')).toEqual([]);
	});

	it('lists a handful at most', () => {
		const many = Array.from({ length: 20 }, (_, i): MentionOption => ({ kind: 'file', name: `f${i}.ts`, where: 'src' }));
		expect(filterMentions(many, 'f')).toHaveLength(MAX_MENTION_OPTIONS);
	});
});

describe('insertMention', () => {
	it('swaps the query for the name and a space, caret after it', () => {
		expect(insertMention('ask @Ko now', 4, 7, 'Koda')).toEqual({ text: 'ask @Koda now', caret: 10 });
		expect(insertMention('@d', 0, 2, 'Dana')).toEqual({ text: '@Dana ', caret: 6 });
	});
});
