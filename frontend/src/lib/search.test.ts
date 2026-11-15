import { describe, expect, it } from 'vitest';
import { isSearchShortcut, searchHref, searchUrl } from './search';
import type { SearchResult } from '$lib/types';

const key = (key: string, extra: Partial<KeyboardEvent> = {}) => ({ key, metaKey: false, ctrlKey: false, altKey: false, shiftKey: false, ...extra });

describe('search', () => {
	it('opens on ⌘K or Ctrl+K only', () => {
		expect(isSearchShortcut(key('k', { metaKey: true }))).toBe(true);
		expect(isSearchShortcut(key('K', { ctrlKey: true }))).toBe(true);
		expect(isSearchShortcut(key('k'))).toBe(false);
		expect(isSearchShortcut(key('k', { metaKey: true, shiftKey: true }))).toBe(false);
	});

	it('asks for the trimmed query, naming kinds only when narrowed', () => {
		expect(searchUrl('  ')).toBeNull();
		expect(searchUrl(' kopi ')).toBe('/search?q=kopi&limit=20');
		expect(searchUrl('a&b', ['memory'], 5)).toBe('/search?q=a%26b&limit=5&kinds=memory');
	});

	it('opens each hit where it lives', () => {
		const hit = (extra: Partial<SearchResult>): SearchResult => ({ kind: 'chat', id: 'x', title: '', snippet: null, updated_at: '', ...extra });
		expect(searchHref(hit({ session_id: 's1' }))).toBe('/chat/s1');
		expect(searchHref(hit({ session_id: 's1', project_id: 'p1' }))).toBe('/projects/session/s1');
		expect(searchHref(hit({ kind: 'memory' }))).toBe('/memory');
		expect(searchHref(hit({ kind: 'file', session_id: 's2', project_id: 'p2' }))).toBe('/projects/session/s2');
	});
});
