import { describe, expect, it } from 'vitest';
import { pinnedFirst, searchChats } from './sessionOrder';

const chat = (id: string, pinned_at: string | null = null) => ({ id, pinned_at });

describe('pinnedFirst', () => {
	it('puts pinned chats on top, newest pin first, and keeps the rest in their order', () => {
		const chats = [chat('a'), chat('b', '2026-10-01T00:00:00Z'), chat('c'), chat('d', '2026-10-20T00:00:00Z')];
		expect(pinnedFirst(chats).map((c) => c.id)).toEqual(['d', 'b', 'a', 'c']);
	});

	it('leaves a list with nothing pinned as it was', () => {
		const chats = [chat('a'), chat('b')];
		expect(pinnedFirst(chats)).toEqual(chats);
	});

	it('takes a pin with no time as pinned all the same', () => {
		expect(pinnedFirst([{ id: 'a', pinned_at: null }, { id: 'b', pinned_at: null, pinned: true }]).map((c) => c.id)).toEqual(['b', 'a']);
	});
});

describe('searchChats', () => {
	const chats = [
		{ title: 'Budget belanja Oktober', last_message: { content: 'Rp 2,1 jt on food' } },
		{ title: null, last_message: { content: 'Remind me to stretch' } },
		{ title: 'Bandung weekend', last_message: null },
	];

	it('finds every word, in the title or the last message, whatever the case', () => {
		expect(searchChats(chats, 'oktober FOOD')).toEqual([chats[0]]);
		expect(searchChats(chats, 'stretch')).toEqual([chats[1]]);
		expect(searchChats(chats, '  ')).toEqual(chats);
		expect(searchChats(chats, 'lombok')).toEqual([]);
	});
});
