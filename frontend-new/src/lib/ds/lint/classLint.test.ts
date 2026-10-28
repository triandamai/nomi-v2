import { describe, expect, it } from 'vitest';
import { classNames, findClassProblems } from './classLint';

// The design system's "Don't" list, checked on every component: raw colours, the dark: variant,
// shadows on flat things, arbitrary values and classes Tailwind doesn't know (which do nothing,
// silently, because only the design's tokens exist).

const known = new Set(['flex', 'gap-2', 'bg-surface', 'text-text-2', 'shadow-float', 'rounded-sm', 'hover:bg-surface-2', 'sr-only']);
const isKnown = (name: string) => known.has(name);

describe('classNames', () => {
	it('reads static classes, class directives and string literals in class expressions', () => {
		const source = `
			<div class="flex gap-2 {open ? 'x' : 'y'}"></div>
			<button class:bg-surface={active} class={['rounded-sm', active && "text-text-2"]}></button>
			<span class='sr-only'></span>`;
		expect(classNames(source).sort()).toEqual(['bg-surface', 'flex', 'gap-2', 'rounded-sm', 'sr-only', 'text-text-2', 'x', 'y'].sort());
	});

	it('ignores script and style blocks', () => {
		expect(classNames('<script>const c = "class=\\"oops\\"";</script><style>.a { color: red; }</style><p class="flex"></p>')).toEqual(['flex']);
	});
});

describe('findClassProblems', () => {
	it('accepts design-token utilities', () => {
		expect(findClassProblems('<p class="flex gap-2 bg-surface hover:bg-surface-2 shadow-float"></p>', isKnown)).toEqual([]);
	});

	it('rejects raw colours in classes and inline styles', () => {
		expect(findClassProblems('<p class="bg-[#fff]"></p>', isKnown)).toContain('raw colour: bg-[#fff]');
		expect(findClassProblems('<p style="color: #FF0000"></p>', isKnown)).toContain('raw colour in style: #FF0000');
	});

	it('rejects the dark: variant, because the tokens carry the theme', () => {
		expect(findClassProblems('<p class="dark:bg-surface"></p>', isKnown)).toContain('dark: variant: dark:bg-surface');
	});

	it('rejects shadows other than the floating one', () => {
		expect(findClassProblems('<p class="shadow-lg"></p>', () => true)).toContain('shadow on a flat element: shadow-lg');
	});

	it('rejects arbitrary values', () => {
		expect(findClassProblems('<p class="p-[13px]"></p>', () => true)).toContain('arbitrary value: p-[13px]');
	});

	it('rejects classes Tailwind generates nothing for', () => {
		expect(findClassProblems('<p class="bg-surfce"></p>', isKnown)).toContain('unknown class: bg-surfce');
	});
});
