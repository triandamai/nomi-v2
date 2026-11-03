import { describe, expect, it } from 'vitest';
import { codeTree, quoteLines } from './codeTree';

describe('codeTree', () => {
	it('lists folders before their files, indented, with the open file marked', () => {
		const tree = codeTree(['src/routes/+page.svelte', 'README.md', 'src/lib/db.sql'], 'src/lib/db.sql');
		expect(tree.map((entry) => [entry.name, entry.depth, entry.kind])).toEqual([
			['README.md', 0, 'text'],
			['src', 0, 'folder'],
			['lib', 1, 'folder'],
			['db.sql', 2, 'database'],
			['routes', 1, 'folder'],
			['+page.svelte', 2, 'file'],
		]);
		expect(tree.find((entry) => entry.active)?.path).toBe('src/lib/db.sql');
	});

	it('quotes picked lines with their file and range', () => {
		const quote = quoteLines('src/app.ts', 'a\nb\nc\nd', 2, 3);
		expect(quote.range).toBe('2–3');
		expect(quote.snippet).toBe('b');
		expect(quote.text).toContain('```\nb\nc\n```');
	});
});
