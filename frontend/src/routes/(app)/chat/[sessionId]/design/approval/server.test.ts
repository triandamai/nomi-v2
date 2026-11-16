import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, json, settle, textResponse } from '../../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { POST } = await import('./+server');

const post = (body: unknown) =>
	POST({ params: { sessionId: 's1' }, request: new Request('http://localhost/chat/s1/design/approval', { method: 'POST', body: JSON.stringify(body) }), cookies: cookieJar(), fetch } as never) as Promise<Response>;

beforeEach(() => apiFetch.mockReset());

describe('design studio: approval', () => {
	it('records the person’s approval of a version', async () => {
		apiFetch.mockResolvedValue(json({ version: 2, note: null, created_at: '2026-11-15T10:00:00Z' }, 201));
		const response = await post({ version: 2 });
		expect(response.status).toBe(201);
		expect(await response.json()).toMatchObject({ version: 2 });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/design/approval');
		expect(apiFetch.mock.calls[0][3]).toMatchObject({ method: 'POST', body: JSON.stringify({ version: 2 }) });
	});

	it('says so when the approval was not taken', async () => {
		apiFetch.mockResolvedValue(textResponse('no such version', 404));
		const outcome = (await settle(() => post({ version: 9 }))) as { thrown?: { status?: number; body?: { message?: string } } };
		expect(outcome.thrown?.status).toBe(404);
		expect(outcome.thrown?.body?.message).toBe('That approval could not be saved. Try again.');
	});
});
