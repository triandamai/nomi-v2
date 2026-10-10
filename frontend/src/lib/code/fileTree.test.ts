import { describe, expect, it } from 'vitest';
import { ancestorFolders, buildFileTree } from './fileTree';

describe('buildFileTree', () => {
	it('nests files under their folders, folders first', () => {
		const tree = buildFileTree(['package.json', 'src/routes/+page.svelte', 'src/app.html', 'src/lib/db.ts', 'README.md']);
		expect(tree.map((n) => n.name)).toEqual(['src', 'package.json', 'README.md']);
		const src = tree[0];
		expect(src.kind === 'folder' && src.children.map((n) => n.name)).toEqual(['lib', 'routes', 'app.html']);
		const routes = src.kind === 'folder' ? src.children[1] : null;
		expect(routes?.kind === 'folder' && routes.children[0]).toEqual({ kind: 'file', name: '+page.svelte', path: 'src/routes/+page.svelte' });
	});
});

describe('ancestorFolders', () => {
	it('lists each parent folder', () => {
		expect(ancestorFolders('src/lib/server/db.ts')).toEqual(['src', 'src/lib', 'src/lib/server']);
		expect(ancestorFolders('index.html')).toEqual([]);
	});
});
