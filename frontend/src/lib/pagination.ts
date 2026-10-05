/** Pages needed for `total` items at `perPage` each; never fewer than one. */
export function pageCount(total: number, perPage: number): number {
	return Math.max(1, Math.ceil(total / Math.max(1, perPage)));
}

/** `page` kept within 1..pages; anything unreadable is page 1. */
export function clampPage(page: number, pages: number): number {
	if (!Number.isFinite(page)) return 1;
	return Math.min(Math.max(1, Math.trunc(page)), Math.max(1, pages));
}

/** The items on `page` (1-based). */
export function pageSlice<T>(items: T[], page: number, perPage: number): T[] {
	const start = (page - 1) * perPage;
	return items.slice(start, start + perPage);
}

/**
 * The page numbers to offer: the first, the last, and the current one with its neighbours,
 * with 'gap' where numbers are skipped. 1 … 4 5 6 … 10
 */
export function pageWindow(page: number, pages: number): (number | 'gap')[] {
	const shown = new Set([1, pages, page - 1, page, page + 1].filter((p) => p >= 1 && p <= pages));
	// A gap hiding a single page shows that page instead.
	for (const p of [...shown]) {
		if (shown.has(p + 2) && !shown.has(p + 1)) shown.add(p + 1);
	}
	const sorted = [...shown].sort((a, b) => a - b);
	const out: (number | 'gap')[] = [];
	sorted.forEach((p, i) => {
		if (i > 0 && p - sorted[i - 1] > 1) out.push('gap');
		out.push(p);
	});
	return out;
}
