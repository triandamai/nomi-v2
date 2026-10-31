import { slugify } from '../slug.ts';

// How the components relate, read from their imports: what each is built from, what is built
// from it, and which sections of the three design sheets show it (the gallery's section files,
// through any helper components they use). Takes sources keyed by their path in src/lib; answers
// with paths in src/lib/ds, as the docs name components (actions/Button.svelte).

export type SheetKey = 'nomi' | 'chat' | 'build';
export type SectionRef = { sheet: SheetKey; title: string; id: string };
export type Relations = { uses: string[]; usedBy: string[]; sections: SectionRef[] };

const isComponent = (path: string) => path.startsWith('ds/') && path.endsWith('.svelte') && !path.endsWith('Fixture.svelte');

function resolve(from: string, specifier: string): string | undefined {
	if (specifier.startsWith('$lib/')) return specifier.slice('$lib/'.length);
	if (!specifier.startsWith('.')) return undefined;
	const parts = from.split('/').slice(0, -1);
	for (const part of specifier.split('/')) {
		if (part === '..') parts.pop();
		else if (part !== '.') parts.push(part);
	}
	return parts.join('/');
}

const imports = (path: string, source: string) =>
	[...source.matchAll(/^\s*import\s+[\w{},\s*]+\s+from\s+['"]([^'"]+\.svelte)['"]/gm)].map(([, specifier]) => resolve(path, specifier)).filter((target): target is string => !!target);

const sheetOf = (path: string): SheetKey => (path.startsWith('gallery/chat/') ? 'chat' : path.startsWith('gallery/build/') ? 'build' : 'nomi');

export function relations(files: Map<string, string>): Record<string, Relations> {
	const result: Record<string, Relations> = {};
	for (const path of [...files.keys()].filter(isComponent).sort()) result[path] = { uses: [], usedBy: [], sections: [] };

	for (const path of Object.keys(result)) {
		const uses = [...new Set(imports(path, files.get(path)!).filter((target) => target in result && target !== path))].sort();
		result[path].uses = uses;
		for (const target of uses) result[target].usedBy.push(path);
	}

	// Every gallery file reaches some components; a section file is one with a GallerySection title
	// (or, for the few drawn by hand, a top-level labelled <section>).
	const gallery = [...files.keys()].filter((path) => path.startsWith('gallery/') && path.endsWith('.svelte'));
	const reach = (path: string, seen = new Set<string>()): string[] => {
		if (seen.has(path)) return [];
		seen.add(path);
		return imports(path, files.get(path) ?? '').flatMap((target) => (target in result ? [target] : target.startsWith('gallery/') ? reach(target, seen) : []));
	};
	for (const path of gallery) {
		const source = files.get(path)!;
		const title = source.match(/<GallerySection\b[^>]*?\btitle="([^"]+)"/)?.[1] ?? source.match(/^<section\b[^>]*?\baria-label="([^"]+)"/m)?.[1];
		if (!title) continue;
		const section = { sheet: sheetOf(path), title, id: slugify(title) };
		for (const component of new Set(reach(path))) result[component].sections.push(section);
	}
	const inDs = (path: string) => path.slice('ds/'.length);
	return Object.fromEntries(
		Object.entries(result).map(([path, { uses, usedBy, sections }]) => [inDs(path), { uses: uses.map(inDs), usedBy: usedBy.map(inDs), sections }]),
	);
}
