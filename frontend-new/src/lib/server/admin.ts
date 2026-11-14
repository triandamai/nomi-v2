import { error, fail, isRedirect } from '@sveltejs/kit';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import type { Problem } from '$lib/admin/api';

export type { Problem };

// The admin console's calls to the backend's /api/admin/*: every load and action goes through
// apiFetch (the person's own sign-in) and reads a refusal with readApiError, keeping the backend's
// own reason to show. A sign-in that can't be renewed still goes to sign in (apiFetch's redirect).

export type Loaded<T> = { ok: true; data: T } | { ok: false; problem: Problem };

async function call<T>(event: ApiEvent, path: string, init?: RequestInit): Promise<Loaded<T>> {
	let response: Response;
	try {
		response = await apiFetch(event, path, init);
	} catch (thrownError) {
		if (isRedirect(thrownError)) throw thrownError;
		// The backend couldn't be reached.
		return { ok: false, problem: { status: 503, message: '' } };
	}
	if (!response.ok) {
		const said = await readApiError(response);
		return {
			ok: false,
			problem: { status: said.status, message: said.message, ...(said.code && { code: said.code }), ...(said.models && { models: said.models }) },
		};
	}
	const text = await response.text().catch(() => '');
	if (!text) return { ok: true, data: null as T };
	try {
		return { ok: true, data: JSON.parse(text) as T };
	} catch {
		return { ok: true, data: text as T };
	}
}

/** `GET path`. */
export const getJson = <T>(event: ApiEvent, path: string): Promise<Loaded<T>> => call<T>(event, path);

/** A change: `method path`, with a JSON body when there is one. */
export function send<T = unknown>(event: ApiEvent, path: string, method: 'POST' | 'PUT' | 'PATCH' | 'DELETE', body?: unknown): Promise<Loaded<T>> {
	return call<T>(event, path, { method, ...(body !== undefined && { body: JSON.stringify(body) }) });
}

/** A page's own data: when the backend refuses the area, the console shows its no-access page. */
export function mustLoad<T>(loaded: Loaded<T>): Loaded<T> {
	if (!loaded.ok && loaded.problem.status === 403) error(403, { message: loaded.problem.message });
	return loaded;
}

/** What a form action answers, named so the page knows which form it was. */
export type ActionDone<T = unknown> = { action: string; ok: true; data: T };
export type ActionFailed = { action: string; ok: false; problem: Problem };

/** Runs one change for a form action: its answer, or the form failed with the backend's reason. */
export async function act<T = unknown>(event: ApiEvent, action: string, path: string, method: 'POST' | 'PUT' | 'PATCH' | 'DELETE', body?: unknown) {
	const result = await send<T>(event, path, method, body);
	if (result.ok) return { action, ok: true as const, data: result.data };
	return fail(result.problem.status >= 400 ? result.problem.status : 500, { action, ok: false as const, problem: result.problem });
}

/** A form's own refusal, before anything is sent (a missing name, a number that isn't one). */
export function refuse(action: string, message: string, extra: Record<string, unknown> = {}) {
	return fail(400, { action, ok: false as const, problem: { status: 400, message, code: 'invalid' }, ...extra });
}

/** A form field, trimmed; empty when it isn't there. */
export function field(form: FormData, name: string): string {
	const value = form.get(name);
	return typeof value === 'string' ? value.trim() : '';
}
