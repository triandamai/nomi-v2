import { isRedirect } from '@sveltejs/kit';
import type { SearchHit } from '$lib/search';
import { apiFetch, type ApiEvent } from './api';

/** What matches `query` in the person's chats, memory and project files; [] for nothing typed, null when it couldn't look. */
export async function searchAll(event: ApiEvent, query: string): Promise<SearchHit[] | null> {
	const q = query.trim();
	if (!q) return [];
	const params = new URLSearchParams({ q, kinds: 'chat,memory,file', limit: '20' });
	try {
		const response = await apiFetch(event, `/api/search?${params}`);
		return response.ok ? ((await response.json()) as { results: SearchHit[] }).results : null;
	} catch (error) {
		if (isRedirect(error)) throw error;
		return null;
	}
}
