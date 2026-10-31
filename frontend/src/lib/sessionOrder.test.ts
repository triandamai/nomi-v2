import { describe, expect, it } from 'vitest';
import { pinnedFirst } from './sessionOrder';

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
});
