import { resolve, sep } from 'node:path';
import type { Plugin } from 'vite';

// The static docs copy (DS_STATIC=1) serves only /design-system, but SvelteKit builds every route
// of the app into it, and since the copy is one bundle (to stay under share hosts' file cap), every
// page of the app was code each docs page downloaded and parsed before it could respond. In that
// build only, each route outside /design-system is an empty page: its file stays, its code doesn't.

/** Whether a route file is one the docs don't serve: under src/routes, outside design-system, not the root layout. */
export function outsideDocs(file: string, routes: string): boolean {
	if (!file.startsWith(routes)) return false;
	const route = file.slice(routes.length);
	return route.includes(sep) && !route.startsWith(`design-system${sep}`);
}

export function docsOnlyRoutes(root = process.cwd()): Plugin {
	const routes = resolve(root, 'src/routes') + sep;
	return {
		name: 'docs-only-routes',
		enforce: 'pre',
		load(id) {
			const file = id.split('?')[0];
			if (id.includes('?') || !outsideDocs(file, routes)) return;
			if (file.endsWith('.svelte')) return '<script lang="ts"></script>';
			if (/\.(ts|js)$/.test(file)) return 'export {};';
		},
	};
}
