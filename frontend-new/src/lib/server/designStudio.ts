import { isRedirect } from '@sveltejs/kit';
import { apiFetch, type ApiEvent } from './api';
import { readApiError } from './apiError';
import type { Outcome } from './chat';
import type { FeedbackDraft, StudioApproval, StudioFeedback, StudioPublishOutcome, StudioResponse } from '$lib/design/designStudio';

/** A backend that can't be reached answers like one that's down. */
async function call(event: ApiEvent, path: string, init?: RequestInit): Promise<Response> {
	try {
		return await apiFetch(event, path, init);
	} catch (error) {
		if (isRedirect(error)) throw error;
		return new Response('', { status: 503 });
	}
}

async function failed(response: Response): Promise<{ ok: false; status: number; message: string }> {
	return { ok: false, status: response.status, message: (await readApiError(response)).message };
}

/** A chat's Design Studio as the person may see it: its directions, previews, feedback, and approval. */
export async function loadStudio(event: ApiEvent, sessionId: string): Promise<{ status: number; studio: StudioResponse | null }> {
	const response = await call(event, `/api/sessions/${encodeURIComponent(sessionId)}/design`);
	if (!response.ok) return { status: response.status, studio: null };
	const { directions, previews, feedback, approval } = (await response.json().catch(() => ({}))) as Partial<StudioResponse>;
	return {
		status: response.status,
		studio: {
			directions: directions ?? null,
			previews: previews ?? [],
			feedback: feedback ?? [],
			approval: approval ?? null,
		},
	};
}

/** Saves a feedback pin or note on a design preview version. */
export async function addStudioFeedback(event: ApiEvent, sessionId: string, draft: FeedbackDraft | Record<string, unknown>): Promise<Outcome<StudioFeedback>> {
	const response = await call(event, `/api/sessions/${encodeURIComponent(sessionId)}/design/feedback`, {
		method: 'POST',
		body: JSON.stringify(draft),
	});
	if (!response.ok) return failed(response);
	return { ok: true, data: (await response.json()) as StudioFeedback };
}

/** Approves a design preview version so Sora hands it off to Koda. */
export async function approveStudioVersion(event: ApiEvent, sessionId: string, version: number, note?: string | null): Promise<Outcome<StudioApproval>> {
	const response = await call(event, `/api/sessions/${encodeURIComponent(sessionId)}/design/approval`, {
		method: 'POST',
		body: JSON.stringify({ version, ...(note ? { note } : {}) }),
	});
	if (!response.ok) return failed(response);
	return { ok: true, data: (await response.json()) as StudioApproval };
}

/** Publishes a design preview version outside Nomi again after an earlier publish failure. */
export async function publishStudioVersion(event: ApiEvent, sessionId: string, version: number): Promise<Outcome<StudioPublishOutcome>> {
	const response = await call(event, `/api/sessions/${encodeURIComponent(sessionId)}/design/publish`, {
		method: 'POST',
		body: JSON.stringify({ version }),
	});
	if (!response.ok) return failed(response);
	return { ok: true, data: (await response.json()) as StudioPublishOutcome };
}
