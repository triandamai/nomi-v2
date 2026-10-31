import { readFileSync, readdirSync } from 'node:fs';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// The Docker image builds from the folder minus .dockerignore. Code that imports a left-out file
// builds here and fails there, so catch it before the image build does.
const root = fileURLToPath(new URL('../..', import.meta.url));
const ignored = readFileSync(join(root, '.dockerignore'), 'utf8')
	.split('\n')
	.map((line) => line.trim())
	.filter((line) => line && !line.startsWith('#') && !line.startsWith('!'));
const isIgnored = (path: string) => ignored.some((pattern) => path === pattern || path.startsWith(`${pattern}/`) || (pattern.endsWith('.*') && path.startsWith(pattern.slice(0, -1))));

const sources = (dir: string): string[] =>
	readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const path = join(dir, entry.name);
		if (entry.isDirectory()) return entry.name === 'paraglide' ? [] : sources(path);
		return /\.(svelte|ts|js)$/.test(entry.name) && !/\.test\.ts$/.test(entry.name) ? [path] : [];
	});

describe('the Docker image', () => {
	it('has every file the app imports', () => {
		const missing = sources(join(root, 'src')).flatMap((file) =>
			[...readFileSync(file, 'utf8').matchAll(/(?:from\s+|import\()\s*['"](\.{1,2}\/[^'"?]+)/g)]
				.map(([, specifier]) => relative(root, resolve(dirname(file), specifier)))
				.filter(isIgnored)
				.map((target) => `${relative(root, file)} → ${target}`),
		);
		expect(missing).toEqual([]);
	});
});
