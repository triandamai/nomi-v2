import { callJson } from './client';

// A new chat from Home or a blank chat (POST /chat): its first message, on the model picked for
// it, and where it went. Offline keeps the draft where it was written; a sign-in that ran out asks
// to sign in again (Session Expired); a refusal says the backend's words.

export type Started = { kind: 'opened'; href: string } | { kind: 'offline' } | { kind: 'signed_out' } | { kind: 'failed'; message: string };

export async function startChat(text: string, model: string | null | undefined, fetchFn: typeof fetch = fetch): Promise<Started> {
	const result = await callJson<{ sessionId: string; sent: boolean }>('/chat', { method: 'POST', body: { text, ...(model && { model }) } }, fetchFn);
	if (result.ok) return { kind: 'opened', href: `/chat/${result.data.sessionId}` };
	if (result.offline) return { kind: 'offline' };
	if (result.signedOut) return { kind: 'signed_out' };
	return { kind: 'failed', message: result.message };
}
