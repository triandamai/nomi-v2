// Code (Flow / Build — Code, terminal & phone preview uR6dq, steps 1–2): the project's files as the
// code view's tree, the file it opens on, and lines quoted into the chat to ask Koda about.

export type TreeEntry = { name: string; depth: number; kind: 'folder' | 'file' | 'database' | 'text'; path: string; active?: boolean };

const kindOf = (name: string): TreeEntry['kind'] => (/\.(sql|sqlite|db)$/i.test(name) ? 'database' : /\.(md|txt)$/i.test(name) ? 'text' : 'file');

type Node = { folders: Map<string, Node>; files: string[] };

/** Folders first, then files, each level by name (case aside); the open file marked. */
export function fileTree(files: { path: string }[], open: string | null): TreeEntry[] {
	const root: Node = { folders: new Map(), files: [] };
	for (const { path } of files) {
		const parts = path.split('/');
		let node = root;
		for (const part of parts.slice(0, -1)) {
			if (!node.folders.has(part)) node.folders.set(part, { folders: new Map(), files: [] });
			node = node.folders.get(part)!;
		}
		node.files.push(parts[parts.length - 1]);
	}
	const byName = (a: string, b: string) => a.localeCompare(b, 'en', { sensitivity: 'base' });
	const walk = (node: Node, depth: number, prefix: string): TreeEntry[] => [
		...[...node.folders.keys()].sort(byName).flatMap((name) => {
			const path = prefix ? `${prefix}/${name}` : name;
			return [{ name, depth, kind: 'folder' as const, path }, ...walk(node.folders.get(name)!, depth + 1, path)];
		}),
		...[...node.files].sort(byName).map((name) => {
			const path = prefix ? `${prefix}/${name}` : name;
			return { name, depth, kind: kindOf(name), path, ...(path === open && { active: true }) };
		}),
	];
	return walk(root, 0, '');
}

const ENTRIES = ['src/routes/+page.svelte', 'src/App.svelte', 'src/App.tsx', 'src/App.vue', 'src/pages/index.astro', 'index.html'];

/** The file the code view opens on: the app's own page where there is one. */
export function firstFile(files: { path: string }[]): string | null {
	return ENTRIES.find((path) => files.some((file) => file.path === path)) ?? files[0]?.path ?? null;
}

/** Lines `from`–`to` (1-based) of a file, as the composer's quote and the text sent with the message. */
export function quoteLines(file: string, lines: string[], from: number, to: number) {
	const numbers = from === to ? `${from}` : `${from}–${to}`;
	const picked = lines.slice(from - 1, to);
	return { file, range: `L${numbers}`, snippet: picked[0] ?? '', text: `About \`${file}\` ${from === to ? 'line' : 'lines'} ${numbers}:\n\`\`\`\n${picked.join('\n')}\n\`\`\`` };
}
