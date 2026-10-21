// The form actions and loads every page that connects providers shares: the Providers pages
// (personal and admin), and the pages with a model picker, which connect one in place.
// Failures carry the API's error code (`fail(status, { error, models })`) so the page can say
// what went wrong with `connectionErrorMessage`.
import { fail, isRedirect, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import {
	connectInputFromForm,
	parseApiError,
	type ConnectionModel,
	type ModelMetadata,
	type ProviderConnection,
	type ProviderDefinition,
} from '$lib/providers';

export const PERSONAL_CONNECTIONS = '/api/llm/connections';
export const ADMIN_CONNECTIONS = '/api/admin/settings/llm/connections';

type Fetch = typeof fetch;

async function failFrom(response: Response) {
	const { error, models } = parseApiError(await response.text());
	return fail(response.status, { error, models });
}

const field = (data: FormData, name: string) => {
	const value = data.get(name);
	return typeof value === 'string' ? value.trim() : '';
};

/** GETs `path` and returns `key` from its JSON; null when the API is down or says no. */
async function loadList<T>(fetchFn: Fetch, cookies: Cookies, path: string, key: string): Promise<T[] | null> {
	try {
		const response = await apiFetch(fetchFn, cookies, path);
		if (!response.ok) return null;
		const body = (await response.json()) as Record<string, unknown>;
		return Array.isArray(body[key]) ? (body[key] as T[]) : null;
	} catch (e) {
		// A 401 that couldn't be refreshed sends them to log in; anything else is "couldn't load".
		if (isRedirect(e)) throw e;
		return null;
	}
}

/** Every provider Nomi knows, in the order to show them. */
export const loadCatalog = (fetchFn: Fetch, cookies: Cookies) =>
	loadList<ProviderDefinition>(fetchFn, cookies, '/api/llm/providers', 'providers');

export const loadConnections = (fetchFn: Fetch, cookies: Cookies, base: string) =>
	loadList<ProviderConnection>(fetchFn, cookies, base, 'connections');

export async function connectProvider(fetchFn: Fetch, cookies: Cookies, base: string, data: FormData) {
	const input = connectInputFromForm(data);
	if (!input) return fail(400, { error: 'unknown_provider', models: [] as string[] });
	const response = await apiFetch(fetchFn, cookies, base, { method: 'POST', body: JSON.stringify(input) });
	if (!response.ok) return failFrom(response);
	return { connection: (await response.json()) as ProviderConnection };
}

export async function disconnectProvider(fetchFn: Fetch, cookies: Cookies, base: string, data: FormData) {
	const id = field(data, 'id');
	if (!id) return fail(400, { error: 'not_found', models: [] as string[] });
	const response = await apiFetch(fetchFn, cookies, `${base}/${encodeURIComponent(id)}`, { method: 'DELETE' });
	if (!response.ok) return failFrom(response);
	return { disconnected: id };
}

/** The models a connection offers right now, asked of the provider with the stored key. */
export async function listConnectionModels(fetchFn: Fetch, cookies: Cookies, base: string, data: FormData) {
	const id = field(data, 'connection_id');
	if (!id) return fail(400, { error: 'not_found', models: [] as string[] });
	const response = await apiFetch(fetchFn, cookies, `${base}/${encodeURIComponent(id)}/models`, { method: 'POST' });
	if (!response.ok) return failFrom(response);
	const body = (await response.json()) as { models?: ConnectionModel[] };
	return { connectionModels: body.models ?? [] };
}

/** What the catalog knows about a provider's models: prices, context and what they open. */
export async function listCatalogModels(fetchFn: Fetch, cookies: Cookies, data: FormData) {
	const providerId = field(data, 'provider_id');
	if (!providerId) return fail(400, { error: 'unknown_provider', models: [] as string[] });
	const response = await apiFetch(fetchFn, cookies, `/api/llm/providers/${encodeURIComponent(providerId)}/models`);
	if (!response.ok) return failFrom(response);
	const body = (await response.json()) as { models?: ModelMetadata[] };
	return { catalogModels: body.models ?? [] };
}
