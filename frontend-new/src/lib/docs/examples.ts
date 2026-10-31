import type { Component } from 'svelte';

// Live examples: src/lib/docs/examples/<component slug>/<nn>-<name>.svelte. Each is rendered as is
// and shown as code, so what people copy is what they see. A first-line comment names it:
// <!-- Variants: one primary per view. -->

export type Example = { title: string; description: string; code: string; component: Component };

// Bundled with the component page rather than one file each: the static copy has to stay within
// what a share host accepts (about 500 files).
const modules = import.meta.glob<{ default: Component }>('./examples/*/*.svelte', { eager: true });
const sources = import.meta.glob<string>('./examples/*/*.svelte', { query: '?raw', import: 'default', eager: true });

export function parseExample(path: string, raw: string): Omit<Example, 'component'> {
	const comment = raw.match(/^\s*<!--\s*([\s\S]*?)\s*-->\s*\n?/);
	const named = path.split('/').pop()!.replace(/\.svelte$/, '').replace(/^\d+-/, '').replace(/-/g, ' ');
	const [title, ...rest] = (comment?.[1] ?? '').split(':');
	const capital = (text: string) => text.charAt(0).toUpperCase() + text.slice(1);
	return {
		title: comment ? title.trim() : capital(named),
		description: capital(rest.join(':').trim()),
		code: (comment ? raw.slice(comment[0].length) : raw).trim(),
	};
}

export function examplesFor(slug: string): Example[] {
	const paths = Object.keys(modules)
		.filter((path) => path.startsWith(`./examples/${slug}/`))
		.sort();
	return paths.map((path) => ({ ...parseExample(path, sources[path]), component: modules[path].default }));
}
