import { existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { guides } from './nav';

const docs = fileURLToPath(new URL('../../routes/design-system/(docs)', import.meta.url));

describe('docs navigation', () => {
	it('links only to pages that exist', () => {
		const missing = guides.flatMap((group) => group.links).filter((link) => !existsSync(`${docs}${link.path.replace(/^\/design-system/, '')}/+page.svelte`));
		expect(missing.map((link) => link.path)).toEqual([]);
	});

	it('has one link per page', () => {
		const paths = guides.flatMap((group) => group.links.map((link) => link.path));
		expect(new Set(paths).size).toBe(paths.length);
	});
});
