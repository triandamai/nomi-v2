import { readdirSync, readFileSync } from 'node:fs';
import { join, relative } from 'node:path';
import type { Plugin } from 'vite';
import { extractApi, type ComponentApi } from './extract.ts';
import { relations, type Relations } from './relations.ts';

// `import api from 'virtual:ds-api'`: every design-system component's props, notes and relations,
// read from src/lib at build time (see extract.ts and relations.ts), keyed by its path in
// src/lib/ds. The docs' props tables come from here, so they can't drift from the code.

export type DsApi = Record<string, ComponentApi & Relations>;

const id = 'virtual:ds-api';
const resolved = `\0${id}`;

const svelteFiles = (dir: string): string[] =>
	readdirSync(dir, { withFileTypes: true }).flatMap((entry) =>
		entry.isDirectory() ? svelteFiles(join(dir, entry.name)) : entry.name.endsWith('.svelte') ? [join(dir, entry.name)] : [],
	);

export function dsApi(): Plugin {
	let lib = '';
	const watched = (file: string) => file.startsWith(join(lib, 'ds')) || file.startsWith(join(lib, 'gallery'));
	return {
		name: 'nomi-ds-api',
		configResolved(config) {
			lib = join(config.root, 'src', 'lib');
		},
		resolveId(source) {
			if (source === id) return resolved;
		},
		load(loadId) {
			if (loadId !== resolved) return;
			const paths = [...svelteFiles(join(lib, 'ds')), ...svelteFiles(join(lib, 'gallery'))];
			const files = new Map(paths.map((path) => [relative(lib, path), readFileSync(path, 'utf8')]));
			for (const path of paths) this.addWatchFile(path);
			const graph = relations(files);
			const api: DsApi = Object.fromEntries(Object.entries(graph).map(([file, links]) => [file, { ...extractApi(files.get(`ds/${file}`)!), ...links }]));
			return `export default ${JSON.stringify(api)};\n`;
		},
		hotUpdate({ file }) {
			// A component changed: the next page load reads its API afresh.
			if (!watched(file)) return;
			const module = this.environment.moduleGraph.getModuleById(resolved);
			if (module) this.environment.moduleGraph.invalidateModule(module);
		},
	};
}
