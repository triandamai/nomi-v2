import { isRedirect } from '@sveltejs/kit';
import type { ConnectionsOverview, GithubStatus } from '$lib/adapters/api';
import { apiFetch, type ApiEvent } from './api';
import type { Outcome } from './chat';

// Settings › Connections' calls to the backend, as the person: what's connected, starting Google
// or GitHub (each answers with where the person signs in), finishing it when they come back, a
// GitHub token or a Hevy key, and disconnecting. Logic and API use from the old app
// (frontend/src/routes/(app)/connections).

/** Where GitHub sends the person back to (a chat that asked for it), kept while they're away. */
export const GITHUB_BACK_COOKIE = 'nm_github_back';

/** `code` is the backend's reason ("key_rejected"), `message` its words. */
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
	const text = await response.text();
	let parsed: unknown = null;
	try {
		parsed = text ? JSON.parse(text) : null;
	} catch {
		parsed = null;
	}
	if (!response.ok) {
		const said = (parsed ?? {}) as { error?: unknown; detail?: unknown };
		const code = typeof said.error === 'string' && /^[a-z_]+$/.test(said.error) ? said.error : undefined;
		const message = typeof said.detail === 'string' && said.detail ? said.detail : typeof said.error === 'string' && !code ? said.error : parsed ? '' : text.trim().slice(0, 200);
		return { ok: false, status: response.status, ...(code && { code }), message };
	}
	return { ok: true, data: parsed as T };
}

export async function loadConnections(event: ApiEvent): Promise<{ overview: ConnectionsOverview | null; github: GithubStatus | null }> {
	const [overview, github] = await Promise.all([read<ConnectionsOverview>(event, '/api/connections'), read<GithubStatus>(event, '/api/connections/github')]);
	return { overview, github };
}

export const startGoogle = (event: ApiEvent, services: string[], resume: string | null) =>
	send<{ url: string }>(event, '/api/connections/google/start', 'POST', { services, resume_session_id: resume });
export const finishGoogle = (event: ApiEvent, code: string, state: string) =>
	send<{ email: string; services: string[]; resume_session_id: string | null }>(event, '/api/connections/google/callback', 'POST', { code, state });
export const disconnectGoogle = (event: ApiEvent) => send<void>(event, '/api/connections/google', 'DELETE');

export const startGithub = (event: ApiEvent) => send<{ url: string }>(event, '/api/connections/github/start', 'POST');
export const finishGithub = (event: ApiEvent, code: string, state: string) => send<GithubStatus>(event, '/api/connections/github/callback', 'POST', { code, state });
export const githubToken = (event: ApiEvent, token: string) => send<GithubStatus>(event, '/api/connections/github', 'PUT', { token });
export const disconnectGithub = (event: ApiEvent) => send<void>(event, '/api/connections/github', 'DELETE');

export const connectHevy = (event: ApiEvent, apiKey: string) => send<unknown>(event, '/api/connections/hevy', 'PUT', { api_key: apiKey });
export const disconnectHevy = (event: ApiEvent) => send<void>(event, '/api/connections/hevy', 'DELETE');

export type { Outcome };
