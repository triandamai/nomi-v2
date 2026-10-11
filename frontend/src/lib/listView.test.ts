import { describe, expect, it } from 'vitest';
import { pageCount, pageOf, searchItems } from './listView';

const chats = ['Lunch budget', 'Rent reminder', 'Cafe website', 'Budget for October'].map((title) => ({ title }));

describe('list search', () => {
	it('keeps items holding every word, in any case', () => {
		expect(searchItems(chats, 'BUDGET', (c) => c.title).map((c) => c.title)).toEqual(['Lunch budget', 'Budget for October']);
		expect(searchItems(chats, 'budget oct', (c) => c.title).map((c) => c.title)).toEqual(['Budget for October']);
	});

	it('keeps everything for an empty query', () => {
		expect(searchItems(chats, '   ', (c) => c.title)).toHaveLength(4);
	});
});

describe('list pages', () => {
	const items = Array.from({ length: 23 }, (_, i) => i);

	it('slices the page asked for', () => {
		expect(pageCount(23, 10)).toBe(3);
		expect(pageOf(items, 3, 10)).toEqual([20, 21, 22]);
	});

	it('clamps a page past the end (after a search narrows the list)', () => {
		expect(pageOf(items, 9, 10)).toEqual([20, 21, 22]);
		expect(pageCount(0, 10)).toBe(1);
		expect(pageOf([], 2, 10)).toEqual([]);
	});
});
