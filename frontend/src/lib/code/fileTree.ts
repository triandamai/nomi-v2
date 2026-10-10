// Project files (flat paths) as a folder tree, sorted the way VS Code's explorer sorts them:
// folders first, then files, each alphabetically (case-insensitive).

export type TreeNode =
	| { kind: 'folder'; name: string; path: string; children: TreeNode[] }
	| { kind: 'file'; name: string; path: string };

export function buildFileTree(paths: string[]): TreeNode[] {
	const root: TreeNode[] = [];
	const folders = new Map<string, TreeNode[]>([['', root]]);

	for (const path of paths) {
		const parts = path.split('/').filter(Boolean);
		let parent = '';
		for (let i = 0; i < parts.length; i++) {
			const name = parts[i];
			const here = parent ? `${parent}/${name}` : name;
			const siblings = folders.get(parent)!;
			if (i === parts.length - 1) {
				siblings.push({ kind: 'file', name, path: here });
			} else if (!folders.has(here)) {
				const children: TreeNode[] = [];
				folders.set(here, children);
				siblings.push({ kind: 'folder', name, path: here, children });
			}
			parent = here;
		}
	}

	const sort = (nodes: TreeNode[]) => {
		nodes.sort((a, b) => (a.kind === b.kind ? a.name.localeCompare(b.name, undefined, { sensitivity: 'base' }) : a.kind === 'folder' ? -1 : 1));
		for (const node of nodes) if (node.kind === 'folder') sort(node.children);
	};
	sort(root);
	return root;
}

/** Every folder on the way to `path` (so opening a file can reveal it in the tree). */
export function ancestorFolders(path: string): string[] {
	const parts = path.split('/').slice(0, -1);
	return parts.map((_, i) => parts.slice(0, i + 1).join('/'));
}
