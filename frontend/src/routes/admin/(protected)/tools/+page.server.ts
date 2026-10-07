import { fail } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { Actions, PageServerLoad } from './$types';
import { m } from '$lib/paraglide/messages';

export type KeySource = 'saved' | 'env' | 'missing';

export type SearchProviderInfo = { id: string; needs_key: boolean; env_var: string; key_source: KeySource };

export type WebSearchSettings = {
	provider: string;
	max_results: number;
	max_results_limit: number;
	base_url: string | null;
	providers: SearchProviderInfo[];
};

export type ReadPageSettings = { max_chars: number };

export type ToolEntry = {
	name: string;
	description: string;
	enabled: boolean;
	required: boolean;
	configurable: boolean;
	ready: boolean;
	used_by: string[];
	settings: WebSearchSettings | ReadPageSettings | null;
};

export type ToolGroup = { key: string; kind: 'crew' | 'agent' | 'custom'; label: string; tools: ToolEntry[] };

export type SearchHit = { title: string; url: string; snippet: string };

export const load: PageServerLoad = async ({ cookies, fetch }) => {
	const response = await apiFetch(fetch, cookies, '/api/admin/tools');
	if (!response.ok) return { groups: null as ToolGroup[] | null };
	const body = (await response.json()) as { groups: ToolGroup[] };
	return { groups: body.groups };
};

async function failure(response: Response, fallback: string) {
	const message = await response.text();
	return fail(response.status, { error: message || fallback });
}

export const actions: Actions = {
	toggle: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const name = data.get('name');
		if (typeof name !== 'string' || !name) return fail(400, { error: m.err_save_tool() });
		const response = await apiFetch(fetch, cookies, `/api/admin/tools/${encodeURIComponent(name)}`, {
			method: 'PUT',
			body: JSON.stringify({ enabled: data.get('enabled') === 'true' }),
		});
		if (!response.ok) return failure(response, m.err_save_tool());
		return { success: true };
	},

	saveSearch: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const text = (key: string) => {
			const value = data.get(key);
			return typeof value === 'string' && value.trim().length > 0 ? value.trim() : null;
		};
		const response = await apiFetch(fetch, cookies, '/api/admin/tools/web_search/settings', {
			method: 'PUT',
			body: JSON.stringify({
				provider: text('provider') ?? 'tavily',
				max_results: Number(text('max_results') ?? 5),
				base_url: text('base_url'),
				api_key: text('api_key'),
				clear_api_key: data.get('clear_api_key') === 'true',
			}),
		});
		if (!response.ok) return failure(response, m.err_save_tool());
		return { saved: 'web_search' };
	},

	saveReadPage: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const response = await apiFetch(fetch, cookies, '/api/admin/tools/read_web_page/settings', {
			method: 'PUT',
			body: JSON.stringify({ max_chars: Number(data.get('max_chars') ?? 12000) }),
		});
		if (!response.ok) return failure(response, m.err_save_tool());
		return { saved: 'read_web_page' };
	},

	test: async ({ request, cookies, fetch }) => {
		const data = await request.formData();
		const response = await apiFetch(fetch, cookies, '/api/admin/tools/web_search/test', {
			method: 'POST',
			body: JSON.stringify({ query: data.get('query') }),
		});
		if (!response.ok) return failure(response, m.err_save_tool());
		const result = (await response.json()) as { ok: boolean; provider: string; results: SearchHit[]; error: string | null };
		return { test: result };
	},
};
