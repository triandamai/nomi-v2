import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// The components in $lib/components/ds are twins of frontend-new's design system: the same names,
// props and events, drawn in Material 3 (docs/superpowers/specs/2026-10-30-design-system-parity-audit.md).
// This keeps them twins: a contract that changes on one side fails here until the other follows.

const TWINS = fileURLToPath(new URL('./components/ds', import.meta.url));
const ORIGINALS = fileURLToPath(new URL('../../../frontend-new/src/lib/ds', import.meta.url));

/** Pure modules both apps keep byte for byte. */
const SHARED_MODULES = ['agents/crew.ts', 'agents/mascots.ts', 'overlays/toasts.svelte.ts', 'navigation/rowStyle.ts', 'actions/copier.svelte.ts'];

/**
 * Sections copied from frontend-new as they are: their Tailwind classes name frontend-new's tokens,
 * which $lib/styles/ds-twins.css points at the M3 roles, and the actions, inputs and overlays they
 * use are this app's M3 twins. Every file in them stays the same as frontend-new's.
 */
const COPIED_SECTIONS = ['build', 'chat', 'conversation', 'mobile'];

function files(dir: string, extension: string): string[] {
	return readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
		const path = join(dir, entry.name);
		if (entry.isDirectory()) return files(path, extension);
		return entry.name.endsWith(extension) && !entry.name.includes('.test.') ? [relative(TWINS, path)] : [];
	});
}

function withoutComments(source: string): string {
	return source.replace(/\/\*[\s\S]*?\*\//g, '').replace(/(^|[^:'"`])\/\/.*$/gm, '$1');
}

/** From `start` to the `;` that ends the statement, skipping nested brackets. */
function statement(source: string, start: number): string {
	let depth = 0;
	for (let at = start; at < source.length; at++) {
		const char = source[at];
		if ('{[(<'.includes(char)) depth++;
		else if ('}])>'.includes(char) && source[at - 1] !== '=') depth--;
		else if (char === ';' && depth <= 0) return source.slice(start, at + 1);
	}
	return source.slice(start);
}

/** A component's contract: its type declarations and its `$props()` destructuring, defaults included. */
function contract(source: string): string {
	const code = withoutComments(source);
	const parts: string[] = [];
	for (const match of code.matchAll(/(?:export )?type \w+ =/g)) parts.push(statement(code, match.index));
	const call = code.indexOf('$props()');
	if (call >= 0) parts.push(code.slice(code.lastIndexOf('let {', call), call + '$props()'.length));
	return parts.join('\n').replace(/\s+/g, ' ').replace(/,\s*([}\]])/g, '$1').trim();
}

describe('design-system twins', () => {
	it.each(SHARED_MODULES)('%s is the same file in both apps', (path) => {
		expect(readFileSync(join(TWINS, path), 'utf8')).toBe(readFileSync(join(ORIGINALS, path), 'utf8'));
	});

	it.each(COPIED_SECTIONS.flatMap((section) => [...files(join(TWINS, section), '.svelte'), ...files(join(TWINS, section), '.ts')]))(
		'%s is copied from frontend-new as it is',
		(path) => {
			expect(readFileSync(join(TWINS, path), 'utf8')).toBe(readFileSync(join(ORIGINALS, path), 'utf8'));
		},
	);

	it.each(files(TWINS, '.svelte'))('%s has the same props as frontend-new', (path) => {
		const twin = contract(readFileSync(join(TWINS, path), 'utf8'));
		const original = contract(readFileSync(join(ORIGINALS, path), 'utf8'));
		expect(twin).toBe(original);
	});
});
