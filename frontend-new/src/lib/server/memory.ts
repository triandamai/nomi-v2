import { isRedirect } from '@sveltejs/kit';
import type { MemoryItem, MemoryProfileData } from '$lib/adapters/api';
import type { MemorySettingsChange, MemorySettingsValues } from '$lib/ds/settings/MemorySettings.svelte';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import type { Outcome } from './chat';

// Settings › Memory & context's calls to the backend, as the person: what the crew remembers,
// correcting one, confirming one the crew asked about, and forgetting one; how it learns and how
// much it carries (the settings), and the profile every agent reads.

async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function send(event: ApiEvent, path: string, method: string, body?: unknown): Promise<Outcome<void>> {
	const response = await call(event, path, { method, ...(body !== undefined && { body: JSON.stringify(body) }) });
	if (!response.ok) return { ok: false, status: response.status, message: (await readApiError(response)).message };
	return { ok: true, data: undefined };
}

/** Null when they can't be read. The vectors the backend sends along are left behind. */
export async function loadMemories(event: ApiEvent): Promise<MemoryItem[] | null> {
	const response = await call(event, '/api/memory');
	if (!response.ok) return null;
	const { memories } = (await response.json()) as { memories: (MemoryItem & { embedding?: unknown })[] };
	return memories.map(({ embedding: _, ...memory }) => memory);
}

export const editMemory = (event: ApiEvent, id: string, content: string) => send(event, `/api/memory/${encodeURIComponent(id)}`, 'PUT', { content });
export const confirmMemory = (event: ApiEvent, id: string) => send(event, `/api/memory/${encodeURIComponent(id)}/confirm`, 'POST');
export const forgetMemory = (event: ApiEvent, id: string) => send(event, `/api/memory/${encodeURIComponent(id)}`, 'DELETE');

async function read<T>(event: ApiEvent, path: string): Promise<T | null> {
	const response = await call(event, path);
	return response.ok ? ((await response.json()) as T) : null;
}

type Saved<T> = { ok: true; data: T } | { ok: false; status: number; code?: string; message: string };

async function put<T>(event: ApiEvent, path: string, body: unknown): Promise<Saved<T>> {
	const response = await call(event, path, { method: 'PUT', body: JSON.stringify(body) });
	if (response.ok) return { ok: true, data: (await response.json()) as T };
	const error = await readApiError(response);
	return { ok: false, status: response.status, code: error.code, message: error.message };
}

export const loadMemorySettings = (event: ApiEvent) => read<MemorySettingsValues>(event, '/api/memory/settings');
export const loadProfile = (event: ApiEvent) => read<MemoryProfileData>(event, '/api/memory/profile');
/** Only what's sent changes; the answer is the settings as they now are. */
export const saveMemorySettings = (event: ApiEvent, change: MemorySettingsChange) => put<MemorySettingsValues>(event, '/api/memory/settings', change);
/** The person's own wording; empty clears it. */
export const saveProfile = (event: ApiEvent, text: string) => put<MemoryProfileData>(event, '/api/memory/profile', { text });

