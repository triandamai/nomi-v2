// How much a file change changed, as the design system's file rows say it ("+12 −3"): lines
// added and removed, counted as a multiset so it stays cheap on big files (a moved line counts
// as unchanged).
export function lineChange(content: string, previous: string | null | undefined): { added: number; removed: number } {
	const after = content ? content.split('\n') : [];
	if (previous === null || previous === undefined) return { added: after.length, removed: 0 };
	const before = previous ? previous.split('\n') : [];
	const left = new Map<string, number>();
	for (const line of before) left.set(line, (left.get(line) ?? 0) + 1);
	let added = 0;
	for (const line of after) {
		const count = left.get(line) ?? 0;
		if (count > 0) left.set(line, count - 1);
		else added++;
	}
	let removed = 0;
	for (const count of left.values()) removed += count;
	return { added, removed };
}

export function changeLabel({ added, removed }: { added: number; removed: number }): string {
	return [added > 0 ? `+${added}` : '', removed > 0 ? `−${removed}` : ''].filter(Boolean).join(' ') || '±0';
}
