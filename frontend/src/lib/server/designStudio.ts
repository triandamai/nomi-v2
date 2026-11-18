import type { Cookies } from '@sveltejs/kit';
import { apiFetch } from '$lib/server/api';
import type { StudioResponse } from '$lib/designStudio';

/** A chat's Design Studio as the person may see it: its previews, what was said about them and the
 * approved version. `null` when the backend refused or couldn't answer (the caller says so). */
export async function loadStudio(fetch: typeof globalThis.fetch, cookies: Cookies, sessionId: string): Promise<{ status: number; studio: StudioResponse | null }> {
	const response = await apiFetch(fetch, cookies, `/api/sessions/${encodeURIComponent(sessionId)}/design`);
	if (!response.ok) return { status: response.status, studio: null };
	const { previews, feedback, approval } = (await response.json().catch(() => ({}))) as Partial<StudioResponse>;
	return { status: response.status, studio: { previews: previews ?? [], feedback: feedback ?? [], approval: approval ?? null } };
}
