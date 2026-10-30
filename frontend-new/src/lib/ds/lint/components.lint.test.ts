import { readFileSync, readdirSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';
import { classNames, findClassProblems } from './classLint';
import { tailwindKnows } from './tailwind';

const src = fileURLToPath(new URL('../../..', import.meta.url));

function svelteFiles(dir: string): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const path = join(dir, entry.name);
		if (entry.isDirectory()) return entry.name === 'paraglide' ? [] : svelteFiles(path);
		return entry.name.endsWith('.svelte') ? [path] : [];
	});
}

describe('every component follows the design system', () => {
	it('uses only design-token classes Tailwind generates', async () => {
		const files = svelteFiles(src);
		const sources = new Map(files.map((file) => [relative(src, file), readFileSync(file, 'utf8')]));
		const isKnown = await tailwindKnows([...sources.values()].flatMap(classNames));

		const problems = [...sources].flatMap(([file, source]) => findClassProblems(source, isKnown).map((problem) => `${file}: ${problem}`));
		expect(files.length).toBeGreaterThan(0);
		expect(problems).toEqual([]);
	});

	it('knows a real class from a typo', async () => {
		const isKnown = await tailwindKnows(['bg-surface', 'bg-surfce', 'text-red-500']);
		expect(isKnown('bg-surface')).toBe(true);
		expect(isKnown('bg-surfce')).toBe(false);
		// Only the design's colours exist.
		expect(isKnown('text-red-500')).toBe(false);
	});
});
