import { describe, expect, it } from 'vitest';
import { cellText, parseCell, rowKey } from './rowEdit';

describe('row edits', () => {
	it('shows cells as editable text', () => {
		expect(cellText(null)).toBe('');
		expect(cellText(3)).toBe('3');
		expect(cellText({ a: 1 })).toBe('{"a":1}');
	});

	it('reads the text back as the cell kind', () => {
		expect(parseCell('', null)).toBeNull();
		expect(parseCell('', 'a')).toBe('');
		expect(parseCell('12', 3)).toBe(12);
		expect(parseCell('twelve', 3)).toBe('twelve');
		expect(parseCell('false', true)).toBe(false);
		expect(parseCell('{"a":2}', { a: 1 })).toEqual({ a: 2 });
		expect(parseCell('{broken', { a: 1 })).toBe('{broken');
		expect(parseCell('hi', null)).toBe('hi');
	});

	it('picks the row out by its primary key', () => {
		expect(rowKey(['id', 'title'], [7, 'a'], ['id'])).toEqual({ id: 7 });
		expect(rowKey(['title'], ['a'], ['id'])).toBeNull();
		expect(rowKey(['id'], [7], [])).toBeNull();
	});
});
