import { ago } from '$lib/adapters/time';

// Search (App / Home's top bar, "Search chats, memory, files ⌘K"), on `GET /api/search`: the
// person's own chats in the workspace (titles and messages), their memory, and the paths of files
// in the projects they can open. Grouped as the backend's kinds, each row opening where it lives.

export type SearchHit = { kind: 'chat' | 'memory' | 'file'; id: string; title: string; snippet: string | null; session_id?: string | null; project_id?: string | null; updated_at: string };
export type SearchRow = { id: string; title: string; snippet?: string; meta: string; href: string };
export type SearchGroup = { kind: SearchHit['kind']; rows: SearchRow[] };

const ORDER: SearchHit['kind'][] = ['chat', 'memory', 'file'];

function hrefOf(hit: SearchHit): string | null {
	if (hit.kind === 'chat') return `/chat/${encodeURIComponent(hit.session_id ?? hit.id)}`;
	if (hit.kind === 'memory') return '/settings/memory';
	return hit.project_id ? `/projects/${encodeURIComponent(hit.project_id)}?open=code&file=${encodeURIComponent(hit.id)}` : null;
}

export function searchGroups(hits: SearchHit[], now: Date): SearchGroup[] {
	return ORDER.map((kind) => ({
		kind,
		rows: hits.flatMap((hit) => {
			const href = hit.kind === kind ? hrefOf(hit) : null;
			return href ? [{ id: `${hit.kind}:${hit.id}:${hit.project_id ?? ''}`, title: hit.title, ...(hit.snippet && { snippet: hit.snippet }), meta: ago(hit.updated_at, now), href }] : [];
		}),
	})).filter((group) => group.rows.length > 0);
}
