import { describe, expect, it } from 'vitest';
import { extractApi } from './extract';

const component = (script: string, markup = '<div></div>') => `<script lang="ts">\n${script}\n</script>\n\n${markup}\n`;

describe('extractApi', () => {
	it('reads a Props type: names, types, optional, defaults and doc comments', () => {
		const api = extractApi(
			component(`
	import type { Component } from 'svelte';

	// Design System / Nomi → Actions. One primary per view.
	// Loading shows real progress in its label.
	type Props = {
		label: string;
		/** ghost-danger: a quiet stop. */
		variant?: 'primary' | 'ghost' | 'ghost-danger';
		// Makes it a link.
		href?: string;
		disabled?: boolean;
	};
	let { label, variant = 'primary', href, disabled = false }: Props = $props();
`),
		);
		expect(api.notes).toBe('Design System / Nomi → Actions. One primary per view. Loading shows real progress in its label.');
		expect(api.props).toEqual([
			{ name: 'label', type: 'string', required: true, bindable: false },
			{ name: 'variant', type: "'primary' | 'ghost' | 'ghost-danger'", required: false, default: "'primary'", bindable: false, doc: 'ghost-danger: a quiet stop.' },
			{ name: 'href', type: 'string', required: false, bindable: false, doc: 'Makes it a link.' },
			{ name: 'disabled', type: 'boolean', required: false, default: 'false', bindable: false },
		]);
		expect(api.rest).toBe(false);
	});

	it('reads an inline type, a JSDoc block before $props, $bindable and a rest element', () => {
		const api = extractApi(
			component(`
	import type { Snippet } from 'svelte';

	/**
	 * Names a group of sidebar rows (Talk, Build, Life).
	 * Written in sentence case; shown in capitals.
	 */
	let { label, value = $bindable(''), open = $bindable(), children, ...rest }: { label: string; value?: string; open?: boolean; children?: Snippet } = $props();
`),
		);
		expect(api.notes).toBe('Names a group of sidebar rows (Talk, Build, Life). Written in sentence case; shown in capitals.');
		expect(api.props).toEqual([
			{ name: 'label', type: 'string', required: true, bindable: false },
			{ name: 'value', type: 'string', required: false, default: "''", bindable: true },
			{ name: 'open', type: 'boolean', required: false, bindable: true },
			{ name: 'children', type: 'Snippet', required: false, bindable: false },
		]);
		expect(api.rest).toBe(true);
	});

	it('follows an interface and intersections, and lists the attributes it passes through', () => {
		const api = extractApi(
			component(`
	interface Base { title: string }
	type Props = Base & {
		count?: number;
		[attribute: \`data-\${string}\`]: unknown;
		[attribute: \`aria-\${string}\`]: unknown;
	};
	let { title, count = 0, ...rest }: Props = $props();
`),
		);
		expect(api.props.map((prop) => prop.name)).toEqual(['title', 'count']);
		expect(api.attributes).toEqual(['data-*', 'aria-*']);
	});

	it('spells out the local types its props use', () => {
		const api = extractApi(
			component(`
	type Option = { value: string; label: string };
	type Unused = { x: number };
	type Props = { options: Option[]; picked?: Option };
	let { options, picked }: Props = $props();
`),
		);
		expect(api.types).toEqual({ Option: '{ value: string; label: string }' });
	});

	it('keeps paragraphs apart and ignores comments after $props', () => {
		const api = extractApi(
			component(`
	// First thought.

	// Second thought.
	let { a }: { a: string } = $props();
	// Not part of the description.
	const b = a;
`),
		);
		expect(api.notes).toBe('First thought.\n\nSecond thought.');
	});

	it('copes with a component without props, and skips the module script', () => {
		const api = extractApi(`<script module lang="ts">\n\texport const x = 1;\n</script>\n<script lang="ts">\n\t// Only a mark.\n</script>\n<svg></svg>`);
		expect(api).toEqual({ notes: 'Only a mark.', props: [], rest: false, attributes: [], types: {} });
		expect(extractApi('<p>static</p>')).toEqual({ notes: '', props: [], rest: false, attributes: [], types: {} });
	});

	it('flattens a long union onto one line', () => {
		const api = extractApi(
			component(`
	let {
		size = 16,
	}: {
		size?:
			| 26
			| 16;
	} = $props();
`),
		);
		expect(api.props[0]).toMatchObject({ name: 'size', type: '26 | 16', default: '16' });
	});
});
