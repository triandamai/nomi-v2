import { describe, expect, it } from 'vitest';
import { cellText, tableCsv, tableTsv } from './table';
import { diffStats, plainLines } from './code';
import { nextOption } from './mention';

// The pure parts of Chat responses → Rich content: what a copied or downloaded table holds, how a
// plain code block splits into lines, what a diff's +/− counts are, and how the mention picker
// moves.

describe('tables', () => {
	const columns = ['Week', 'Groceries', 'vs Sep'];
	const rows = [
		['1–7 Oct', '312.000', { value: '+12%', tone: 'bad' as const }],
		['8–14 Oct', '286.000', { value: '−6%', tone: 'good' as const }],
	];

	it('reads a cell whether it is plain or toned', () => {
		expect(cellText('312.000')).toBe('312.000');
		expect(cellText({ value: '+12%', tone: 'bad' })).toBe('+12%');
	});

	it('copies as tab-separated rows, header and total included', () => {
		expect(tableTsv(columns, rows, ['October', '1.173.000', '+20%'])).toBe(
			'Week\tGroceries\tvs Sep\n1–7 Oct\t312.000\t+12%\n8–14 Oct\t286.000\t−6%\nOctober\t1.173.000\t+20%',
		);
	});

	it('downloads as CSV, quoting values with commas, quotes or line breaks', () => {
		expect(tableCsv(['Name', 'Note'], [['Rp 2,2 jt', 'said "soon"'], ['a', 'two\nlines']])).toBe(
			'Name,Note\n"Rp 2,2 jt","said ""soon"""\na,"two\nlines"',
		);
	});
});

describe('code', () => {
	it('splits plain code into lines of one token each, keeping empty lines', () => {
		expect(plainLines('const a = 1\n\nreturn a')).toEqual([[{ text: 'const a = 1' }], [], [{ text: 'return a' }]]);
	});

	it('counts the lines a diff adds and removes', () => {
		expect(
			diffStats([
				{
					header: '@@ -2,4 +2,6 @@',
					lines: [
						{ kind: 'context', old: 2, new: 2, text: 'a' },
						{ kind: 'del', old: 3, text: 'b' },
						{ kind: 'add', new: 3, text: 'c' },
						{ kind: 'add', new: 4, text: 'd' },
					],
				},
				{ header: '@@ -9 +11 @@', lines: [{ kind: 'add', new: 11, text: 'e' }] },
			]),
		).toEqual({ added: 3, removed: 1 });
	});
});

describe('mention picker', () => {
	it('moves with the arrow keys and wraps around', () => {
		expect(nextOption(0, 'ArrowDown', 4)).toBe(1);
		expect(nextOption(3, 'ArrowDown', 4)).toBe(0);
		expect(nextOption(0, 'ArrowUp', 4)).toBe(3);
		expect(nextOption(2, 'Home', 4)).toBe(0);
		expect(nextOption(1, 'End', 4)).toBe(3);
	});

	it('ignores other keys and empty lists', () => {
		expect(nextOption(2, 'a', 4)).toBe(2);
		expect(nextOption(0, 'ArrowDown', 0)).toBe(0);
	});
});
