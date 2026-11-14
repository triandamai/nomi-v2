import { describe, expect, it } from 'vitest';
import { fileTree, firstFile, quoteLines } from './code';

const files = ['src/routes/+page.svelte', 'package.json', 'src/lib/menu.ts', 'src/lib/db/seed.sql', 'README.md'].map((path) => ({ path }));

describe('the code view’s tree', () => {
	it('lists folders before files, each level by name, with the open file marked', () => {
		expect(fileTree(files, 'src/lib/menu.ts')).toEqual([
			{ name: 'src', depth: 0, kind: 'folder', path: 'src' },
			{ name: 'lib', depth: 1, kind: 'folder', path: 'src/lib' },
			{ name: 'db', depth: 2, kind: 'folder', path: 'src/lib/db' },
			{ name: 'seed.sql', depth: 3, kind: 'database', path: 'src/lib/db/seed.sql' },
			{ name: 'menu.ts', depth: 2, kind: 'file', path: 'src/lib/menu.ts', active: true },
			{ name: 'routes', depth: 1, kind: 'folder', path: 'src/routes' },
			{ name: '+page.svelte', depth: 2, kind: 'file', path: 'src/routes/+page.svelte' },
			{ name: 'package.json', depth: 0, kind: 'file', path: 'package.json' },
			{ name: 'README.md', depth: 0, kind: 'text', path: 'README.md' },
		]);
	});

	it('opens on the app’s own page first', () => {
		expect(firstFile(files)).toBe('src/routes/+page.svelte');
		expect(firstFile([{ path: 'index.html' }, { path: 'a.css' }])).toBe('index.html');
		expect(firstFile([{ path: 'z.txt' }])).toBe('z.txt');
		expect(firstFile([])).toBeNull();
	});
});

describe('asking about lines', () => {
	it('quotes the lines asked about, with the file and their numbers', () => {
		expect(quoteLines('src/lib/menu.ts', ['a', 'b', 'c', 'd'], 2, 3)).toEqual({
			file: 'src/lib/menu.ts',
			range: 'L2–3',
			snippet: 'b',
			text: 'About `src/lib/menu.ts` lines 2–3:\n```\nb\nc\n```',
		});
		expect(quoteLines('index.html', ['<h1>'], 1, 1)).toMatchObject({ range: 'L1', text: 'About `index.html` line 1:\n```\n<h1>\n```' });
	});
});
