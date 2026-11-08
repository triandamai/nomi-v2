import { describe, expect, it } from 'vitest';
import { MAX_SLASH_QUERY, SLASH_BLOCKS, slashQuery, slashResults } from './markdownEditor';

describe('the "/" block menu', () => {
	it('opens on a slash that starts a word', () => {
		expect(slashQuery('/')).toBe('');
		expect(slashQuery('some text /hea')).toBe('hea');
		expect(slashQuery('and/or')).toBeNull();
		expect(slashQuery('/heading 1')).toBeNull();
		expect(slashQuery('/' + 'x'.repeat(MAX_SLASH_QUERY + 1))).toBeNull();
	});

	it('finds blocks by any word of their names', () => {
		const names = (block: string) => ({ h2: ['Heading 2', 'heading'], todo: ['To-do list', 'task'] })[block] ?? [block];
		expect(slashResults('', names)).toEqual(SLASH_BLOCKS);
		expect(slashResults('head', names)).toEqual(['h2']);
		expect(slashResults('do', names)).toEqual(['todo']);
		expect(slashResults('TASK', names)).toEqual(['todo']);
		expect(slashResults('zzz', names)).toEqual([]);
	});
});
