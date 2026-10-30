import { readFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { compile } from '@tailwindcss/node';

const src = fileURLToPath(new URL('../../..', import.meta.url));

/** A class as it appears in a selector: `hover:bg-surface` → `hover\:bg-surface`. */
function selector(name: string): string {
	return '.' + name.replace(/[^a-zA-Z0-9_-]/g, (character) => `\\${character}`);
}

/**
 * Compiles the app's own stylesheet with these candidate classes and reports which ones Tailwind
 * generated a rule for. With only the design's tokens defined, a typo or a stock Tailwind colour
 * generates nothing, which would otherwise fail silently.
 */
export async function tailwindKnows(candidates: string[]): Promise<(name: string) => boolean> {
	const compiler = await compile(readFileSync(`${src}/app.css`, 'utf8'), { base: src, onDependency: () => {} });
	const css = compiler.build([...new Set(candidates)]);
	return (name) => css.includes(selector(name));
}
