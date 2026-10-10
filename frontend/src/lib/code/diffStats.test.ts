import { describe, expect, it } from 'vitest';
import { diffStats, fileBadge } from './diffStats';

describe('diffStats', () => {
	it('counts a new file as all added and a deleted one as all removed', () => {
		expect(diffStats(null, 'a\nb\nc\n')).toEqual({ added: 3, removed: 0 });
		expect(diffStats('a\nb', null)).toEqual({ added: 0, removed: 2 });
	});

	it('counts changed lines in an edit', () => {
		expect(diffStats('a\nb\nc\nd', 'a\nB\nc\nd\ne')).toEqual({ added: 2, removed: 1 });
		expect(diffStats('same', 'same')).toEqual({ added: 0, removed: 0 });
	});
});

describe('fileBadge', () => {
	it('labels common types', () => {
		expect(fileBadge('src/routes/+page.svelte').label).toBe('S');
		expect(fileBadge('src/lib/db.ts').label).toBe('TS');
		expect(fileBadge('Makefile').label).toBe('MAK');
	});
});
