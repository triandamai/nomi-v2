// The project's files as the code view's tree (Build → Code view): folders before their files,
// each indented by its depth, the open file marked.
export type TreeEntry = { name: string; depth: number; kind: 'folder' | 'file' | 'database' | 'text'; active?: boolean; path: string };

const TEXT = /\.(md|txt|json|ya?ml|toml|env\.example)$/i;
const DATABASE = /\.(sql|sqlite|db)$/i;

export function codeTree(paths: string[], active: string | null): TreeEntry[] {
	const entries: TreeEntry[] = [];
	const seen = new Set<string>();
	for (const path of [...paths].sort((a, b) => a.localeCompare(b))) {
		const parts = path.split('/');
		for (let depth = 0; depth < parts.length - 1; depth++) {
			const folder = parts.slice(0, depth + 1).join('/');
			if (seen.has(folder)) continue;
			seen.add(folder);
			entries.push({ name: parts[depth], depth: Math.min(depth, 4), kind: 'folder', path: folder });
		}
		const name = parts[parts.length - 1];
		entries.push({
			name,
			depth: Math.min(parts.length - 1, 4),
			kind: DATABASE.test(name) ? 'database' : TEXT.test(name) ? 'text' : 'file',
			active: path === active,
			path,
		});
	}
	return entries;
}

/** What picked lines say when quoted to Koda: the file, the lines, and the code itself. */
export function quoteLines(path: string, content: string, from: number, to: number): { range: string; snippet: string; text: string } {
	const lines = content.split('\n').slice(from - 1, to);
	const range = from === to ? String(from) : `${from}–${to}`;
	return { range, snippet: (lines[0] ?? '').trim(), text: `About \`${path}\` line${from === to ? '' : 's'} ${range}:\n\`\`\`\n${lines.join('\n')}\n\`\`\`` };
}
