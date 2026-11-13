import { isRedirect } from '@sveltejs/kit';
import type { ModelOptions, OwnConnection, ProviderDefinition, UsageMonth, UserModels } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';

// Settings › Models & providers' calls to the backend, as the person: the models in their picker
// (by who pays), Nomi's models and Koda's, their own connections, this month's usage, and every
// provider they can connect; the defaults, what's shown, connecting (the backend checks the key
// with the provider first) and disconnecting. Logic and API use from the old app
// (frontend/src/lib/server/providers.ts, frontend/src/routes/(app)/models).

export type ModelsData = {
	options: ModelOptions;
	models: UserModels | null;
	connections: OwnConnection[];
	usage: UsageMonth | null;
	catalog: ProviderDefinition[];
};

/** `code` is the backend's reason ("key_rejected", "unreachable"), `message` its words. */
export type Sent<T> = { ok: true; data: T } | { ok: false; status: number; code?: string; message: string };

async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function read<T>(event: ApiEvent, path: string): Promise<T | null> {
	const response = await call(event, path);
	return response.ok ? ((await response.json()) as T) : null;
}

async function send<T>(event: ApiEvent, path: string, method: string, body?: unknown): Promise<Sent<T>> {
	const response = await call(event, path, { method, ...(body !== undefined && { body: JSON.stringify(body) }) });
	if (!response.ok) {
		// The provider routes answer { error: "key_rejected", detail: "<the provider's words>" }.
		const said = (await response
			.clone()
			.json()
			.catch(() => null)) as { error?: unknown; detail?: unknown } | null;
		const error = await readApiError(response);
		const message = typeof said?.detail === 'string' && said.detail ? said.detail : error.message;
		return { ok: false, status: response.status, ...(error.code && { code: error.code }), message };
	}
	const text = await response.text();
	return { ok: true, data: (text ? JSON.parse(text) : undefined) as T };
}

/** Null without the picker's models; the rest is left out (or empty) when it can't be read. */
export async function loadModels(event: ApiEvent): Promise<ModelsData | null> {
	const [options, models, connections, usage, catalog] = await Promise.all([
		read<ModelOptions>(event, '/api/llm/model-options'),
		read<UserModels>(event, '/api/llm/models'),
		read<{ connections: OwnConnection[] }>(event, '/api/llm/connections'),
		read<UsageMonth>(event, '/api/usage'),
		read<{ providers: ProviderDefinition[] }>(event, '/api/llm/providers'),
	]);
	if (!options) return null;
	return { options, models, connections: connections?.connections ?? [], usage, catalog: catalog?.providers ?? [] };
}

export const setDefaultModel = (event: ApiEvent, key: string) => send<unknown>(event, '/api/llm/default-model', 'PUT', { key });

export const setCodingModel = (event: ApiEvent, selection: UserModels['coding']) => send<unknown>(event, '/api/llm/coding-selection', 'PUT', selection);

export const setShown = (event: ApiEvent, changes: { key: string; enabled: boolean }[]) => send<unknown>(event, '/api/llm/model-options', 'PUT', { changes });

/** Connects a provider; the backend tries the key (or the address) with it first. */
export const connectProvider = (event: ApiEvent, input: { provider_id: string; api_key?: string; base_url?: string }) => send<OwnConnection>(event, '/api/llm/connections', 'POST', input);

/** The models a connection offers right now, asked of the provider with its stored key. */
export const connectionModels = (event: ApiEvent, id: string) => send<{ models: { id: string; label: string | null }[] }>(event, `/api/llm/connections/${encodeURIComponent(id)}/models`, 'POST');

export const disconnectProvider = (event: ApiEvent, id: string) => send<void>(event, `/api/llm/connections/${encodeURIComponent(id)}`, 'DELETE');
