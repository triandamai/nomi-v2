import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { mcpError } from '$lib/mcpServers';
import type { CatalogApp, ConnectionsOverview, GithubStatus, McpServers } from '$lib/types';
import type { Actions, PageServerLoad } from './$types';

export const load: PageServerLoad = async ({ cookies, fetch, url }) => {
	const read = async <T>(path: string): Promise<T | null> => {
		const response = await apiFetch(fetch, cookies, path);
		return response.ok ? ((await response.json()) as T) : null;
	};
	const [overview, catalog, mcp, github] = await Promise.all([
		read<ConnectionsOverview>('/api/connections'),
		read<CatalogApp[]>('/api/connections/catalog'),
		read<McpServers>('/api/connections/mcp'),
		read<GithubStatus>('/api/connections/github'),
	]);
	return {
		overview,
		catalog: catalog ?? [],
		mcp: mcp ?? { servers: [], callable: false },
		github,
		query: url.searchParams.get('q') ?? '',
	};
};

export const actions: Actions = {
	/** Adds a remote MCP server by its address (never a command). */
	addMcp: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const name = String(data.get('name') ?? '').trim();
		const url = String(data.get('url') ?? '').trim();
		const response = await apiFetch(fetch, cookies, '/api/connections/mcp', { method: 'POST', body: JSON.stringify({ name, url }) });
		if (!response.ok) {
			const code = ((await response.json().catch(() => ({}))) as { error?: string }).error;
			return fail(response.status, { mcpError: mcpError(code), name, url });
		}
		return { mcpAdded: true };
	},
	removeMcp: async ({ request, cookies, fetch }) => {
		const id = String((await request.formData()).get('id') ?? '');
		if (!id) return fail(400, { mcpError: mcpError(undefined) });
		const response = await apiFetch(fetch, cookies, `/api/connections/mcp/${encodeURIComponent(id)}`, { method: 'DELETE' });
		if (!response.ok && response.status !== 404) return fail(response.status, { mcpError: mcpError(undefined) });
		return { mcpRemoved: id };
	},
};
