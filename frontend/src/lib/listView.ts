// Searching, paging and the table/cards choice for the Chats and Projects lists.

export type ListLayout = 'table' | 'cards';

/** The items whose text (from `text`) holds every word of `query`, case-insensitively. */
export function searchItems<T>(items: T[], query: string, text: (item: T) => string): T[] {
	const words = query.toLowerCase().split(/\s+/).filter(Boolean);
	if (words.length === 0) return items;
	return items.filter((item) => {
		const haystack = text(item).toLowerCase();
		return words.every((word) => haystack.includes(word));
	});
}

/** How many pages `count` items make (at least one). */
export function pageCount(count: number, pageSize: number): number {
	return Math.max(1, Math.ceil(count / pageSize));
}

/** Page `page` (from 1, clamped to the pages there are) of `items`. */
export function pageOf<T>(items: T[], page: number, pageSize: number): T[] {
	const last = pageCount(items.length, pageSize);
	const current = Math.min(Math.max(1, page), last);
	return items.slice((current - 1) * pageSize, current * pageSize);
}

const STORAGE_PREFIX = 'nomi.layout.';

/** The layout this browser last chose for list `key`. */
export function storedLayout(key: string, fallback: ListLayout): ListLayout {
	try {
		const value = localStorage.getItem(STORAGE_PREFIX + key);
		return value === 'table' || value === 'cards' ? value : fallback;
	} catch {
		return fallback;
	}
}

export function storeLayout(key: string, layout: ListLayout): void {
	try {
		localStorage.setItem(STORAGE_PREFIX + key, layout);
	} catch {
		// Private mode or blocked storage: the choice just isn't remembered.
	}
}
