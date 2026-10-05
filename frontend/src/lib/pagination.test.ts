import { describe, expect, it } from 'vitest';
import { clampPage, pageCount, pageSlice, pageWindow } from './pagination';

describe('pagination', () => {
	it('counts pages, at least one', () => {
		expect(pageCount(0, 10)).toBe(1);
		expect(pageCount(10, 10)).toBe(1);
		expect(pageCount(11, 10)).toBe(2);
	});

	it('clamps a page into range', () => {
		expect(clampPage(0, 3)).toBe(1);
		expect(clampPage(9, 3)).toBe(3);
		expect(clampPage(Number.NaN, 3)).toBe(1);
	});

	it('slices one page', () => {
		expect(pageSlice([1, 2, 3, 4, 5], 2, 2)).toEqual([3, 4]);
		expect(pageSlice([1, 2, 3, 4, 5], 3, 2)).toEqual([5]);
	});

	it('shows the ends and the neighbourhood, with gaps', () => {
		expect(pageWindow(1, 1)).toEqual([1]);
		expect(pageWindow(1, 4)).toEqual([1, 2, 3, 4]);
		expect(pageWindow(1, 6)).toEqual([1, 2, 'gap', 6]);
		expect(pageWindow(5, 10)).toEqual([1, 'gap', 4, 5, 6, 'gap', 10]);
		expect(pageWindow(3, 10)).toEqual([1, 2, 3, 4, 'gap', 10]);
		expect(pageWindow(10, 10)).toEqual([1, 'gap', 9, 10]);
	});
});
