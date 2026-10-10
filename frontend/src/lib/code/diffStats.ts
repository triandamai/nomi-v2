const splitLines = (s: string) => (s === '' ? [] : s.replace(/\n$/, '').split('\n'));

/** Lines added and removed going from `before` to `after` (either may be absent). */
export function diffStats(before: string | null, after: string | null): { added: number; removed: number } {
	const a = splitLines(before ?? '');
	const b = splitLines(after ?? '');
	// Common lines at both ends don't count; what's left is diffed by longest common subsequence.
	let start = 0;
	while (start < a.length && start < b.length && a[start] === b[start]) start++;
	let endA = a.length;
	let endB = b.length;
	while (endA > start && endB > start && a[endA - 1] === b[endB - 1]) {
		endA--;
		endB--;
	}
	const midA = a.slice(start, endA);
	const midB = b.slice(start, endB);
	// Too big to diff line by line in the browser: count the size difference only.
	if (midA.length * midB.length > 4_000_000) return { added: midB.length, removed: midA.length };
	let prev = new Array<number>(midB.length + 1).fill(0);
	for (let i = 1; i <= midA.length; i++) {
		const row = new Array<number>(midB.length + 1).fill(0);
		for (let j = 1; j <= midB.length; j++) {
			row[j] = midA[i - 1] === midB[j - 1] ? prev[j - 1] + 1 : Math.max(prev[j], row[j - 1]);
		}
		prev = row;
	}
	const common = prev[midB.length];
	return { added: midB.length - common, removed: midA.length - common };
}

/** The short label and colour VS Code-style file tabs use for a path's type. */
export function fileBadge(path: string): { label: string; color: string } {
	const ext = path.split('.').pop()?.toLowerCase() ?? '';
	const name = path.split('/').pop()?.toLowerCase() ?? '';
	if (name === 'package.json') return { label: '{}', color: '#8bc34a' };
	const map: Record<string, { label: string; color: string }> = {
		ts: { label: 'TS', color: '#3178c6' },
		tsx: { label: 'TSX', color: '#3178c6' },
		js: { label: 'JS', color: '#f1dd35' },
		mjs: { label: 'JS', color: '#f1dd35' },
		jsx: { label: 'JSX', color: '#61dafb' },
		svelte: { label: 'S', color: '#ff3e00' },
		vue: { label: 'V', color: '#41b883' },
		astro: { label: 'A', color: '#ff5d01' },
		html: { label: '<>', color: '#e44d26' },
		css: { label: '#', color: '#42a5f5' },
		json: { label: '{}', color: '#f1dd35' },
		md: { label: 'M↓', color: '#519aba' },
		sql: { label: 'SQL', color: '#dad8d8' },
		svg: { label: 'SVG', color: '#ffb13b' }
	};
	return map[ext] ?? { label: ext.slice(0, 3).toUpperCase() || '·', color: '#9d9d9d' };
}
