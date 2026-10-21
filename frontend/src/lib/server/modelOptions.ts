// The model picker on the server: the person's model options, the model a chat uses, the
// default for new chats, and the form actions that change them (shared by the chat, new-chat,
// project chat and Model pages). Loads never break a chat: when the API can't say, the picker
// just has nothing to show.
import { fail, isRedirect, type Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import { connectProvider, PERSONAL_CONNECTIONS } from '$lib/server/providers';
import { parseModelKey, type ModelChange, type ModelChoice, type ModelOptions } from '$lib/modelPicker';
import { m } from '$lib/paraglide/messages';

type Fetch = typeof fetch;

/** GETs `path` as JSON; null when the API is down or says no (a sign-in redirect still goes). */
async function getJson<T>(fetchFn: Fetch, cookies: Cookies, path: string): Promise<T | null> {
	try {
		const response = await apiFetch(fetchFn, cookies, path);
		return response.ok ? ((await response.json()) as T) : null;
	} catch (e) {
		if (isRedirect(e)) throw e;
		return null;
	}
}

/** The person's model options; null when they can't be read (the Model page says so). */
export async function fetchModelOptions(fetchFn: Fetch, cookies: Cookies): Promise<ModelOptions | null> {
	const body = await getJson<Partial<ModelOptions>>(fetchFn, cookies, '/api/llm/model-options');
	if (!body || !Array.isArray(body.groups)) return null;
	return { groups: body.groups, current: body.current ?? null };
}

/** The person's model options for a chat's picker; nothing to offer when they can't be read. */
export async function loadModelOptions(fetchFn: Fetch, cookies: Cookies): Promise<ModelOptions> {
	return (await fetchModelOptions(fetchFn, cookies)) ?? { groups: [], current: null };
}

/** The model this chat replies with; null when it can't be read. */
export async function loadSessionModel(fetchFn: Fetch, cookies: Cookies, sessionId: string): Promise<ModelChoice | null> {
	const body = await getJson<{ model?: ModelChoice | null }>(fetchFn, cookies, `/api/sessions/${sessionId}/model`);
	return body?.model ?? null;
}

/** Shared `setSessionModel` action body: form field `key`. */
export async function saveSessionModel(fetchFn: Fetch, cookies: Cookies, sessionId: string, data: FormData) {
	const key = String(data.get('key') ?? '');
	if (!parseModelKey(key)) return fail(400, { modelError: m.msel_pick_failed() });
	const response = await apiFetch(fetchFn, cookies, `/api/sessions/${sessionId}/model`, { method: 'PUT', body: JSON.stringify({ key }) });
	if (!response.ok) return fail(response.status, { modelError: m.msel_pick_failed() });
	const body = (await response.json()) as { model: ModelChoice };
	return { sessionModel: body.model };
}

/** Shared `setDefaultModel` action body (the Model page): form field `key`, for new chats. */
export async function saveDefaultModel(fetchFn: Fetch, cookies: Cookies, data: FormData) {
	const key = String(data.get('key') ?? '');
	if (!parseModelKey(key)) return fail(400, { modelError: m.msel_pick_failed() });
	const response = await apiFetch(fetchFn, cookies, '/api/llm/default-model', { method: 'PUT', body: JSON.stringify({ key }) });
	if (!response.ok) return fail(response.status, { modelError: m.msel_pick_failed() });
	const body = (await response.json()) as { model: ModelChoice };
	return { defaultModel: body.model };
}

function parseChanges(raw: unknown): ModelChange[] | null {
	if (typeof raw !== 'string') return null;
	let value: unknown;
	try {
		value = JSON.parse(raw);
	} catch {
		return null;
	}
	if (!Array.isArray(value) || value.length === 0) return null;
	const changes: ModelChange[] = [];
	for (const item of value) {
		const { key, enabled } = (item ?? {}) as Record<string, unknown>;
		if (typeof key !== 'string' || !parseModelKey(key) || typeof enabled !== 'boolean') return null;
		changes.push({ key, enabled });
	}
	return changes;
}

/** Shared `setModelVisibility` action body: form field `changes`, JSON `{ key, enabled }[]`. */
export async function saveModelVisibility(fetchFn: Fetch, cookies: Cookies, data: FormData) {
	const changes = parseChanges(data.get('changes'));
	if (!changes) return fail(400, { modelError: m.msel_save_failed() });
	const response = await apiFetch(fetchFn, cookies, '/api/llm/model-options', { method: 'PUT', body: JSON.stringify({ changes }) });
	if (!response.ok) return fail(response.status, { modelError: m.msel_save_failed() });
	return { modelsSaved: true };
}

type ActionEvent = { request: Request; cookies: Cookies; fetch: Fetch };

/** Manage models, wherever the picker is: which models it offers, and connecting a provider in place. */
export const manageModelActions = {
	setModelVisibility: async ({ request, cookies, fetch }: ActionEvent) => saveModelVisibility(fetch, cookies, await request.formData()),
	connect: async ({ request, cookies, fetch }: ActionEvent) => connectProvider(fetch, cookies, PERSONAL_CONNECTIONS, await request.formData()),
};

/** A chat page's picker (a chat, a project's chat): its model, plus Manage models. */
export const sessionModelActions = {
	...manageModelActions,
	setSessionModel: async ({ request, params, cookies, fetch }: ActionEvent & { params: { sessionId: string } }) =>
		saveSessionModel(fetch, cookies, params.sessionId, await request.formData()),
};
