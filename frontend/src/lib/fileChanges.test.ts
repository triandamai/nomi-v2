import { describe, expect, it } from 'vitest';
import { changeLabel, lineChange } from './fileChanges';

describe('lineChange', () => {
	it('counts a new file as all added', () => {
		expect(lineChange('a\nb\nc', null)).toEqual({ added: 3, removed: 0 });
	});
	it('counts lines added and removed', () => {
		expect(lineChange('a\nB\nc\nd', 'a\nb\nc')).toEqual({ added: 2, removed: 1 });
		expect(changeLabel(lineChange('a\nB\nc\nd', 'a\nb\nc'))).toBe('+2 −1');
		expect(changeLabel({ added: 0, removed: 0 })).toBe('±0');
	});
});
