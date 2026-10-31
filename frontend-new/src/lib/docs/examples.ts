import type { Component } from 'svelte';

// Live examples: src/lib/docs/examples/<component slug>/<nn>-<name>.svelte. Each is rendered as is
// and shown as code, so what people copy is what they see. A first-line comment names it:
// <!-- Variants: one primary per view. -->

export type Example = { title: string; description: string; code: string; component: Component };

const modules = import.meta.glob<{ default: Component }>('./examples/*/*.svelte');
const sources = import.meta.glob<string>('./examples/*/*.svelte', { query: '?raw', import: 'default' });

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

export async function examplesFor(slug: string): Promise<Example[]> {
	const paths = Object.keys(modules)
		.filter((path) => path.startsWith(`./examples/${slug}/`))
		.sort();
	return Promise.all(paths.map(async (path) => ({ ...parseExample(path, await sources[path]()), component: (await modules[path]()).default })));
}
