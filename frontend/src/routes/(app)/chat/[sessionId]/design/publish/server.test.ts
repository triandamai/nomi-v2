import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, json, settle, textResponse } from '../../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { POST } = await import('./+server');

const post = (body: unknown) =>
	POST({ params: { sessionId: 's1' }, request: new Request('http://localhost/chat/s1/design/publish', { method: 'POST', body: JSON.stringify(body) }), cookies: cookieJar(), fetch } as never) as Promise<Response>;

beforeEach(() => apiFetch.mockReset());

describe('design studio: publish again', () => {
	it('passes on where the version is now, or why it still is not', async () => {
		apiFetch.mockResolvedValue(json({ version: 2, published_url: null, publish_error: 'Shipyard is down' }));
		const response = await post({ version: 2 });
		expect(response.status).toBe(200);
		expect(await response.json()).toEqual({ version: 2, published_url: null, publish_error: 'Shipyard is down' });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/design/publish');
		expect(apiFetch.mock.calls[0][3]).toMatchObject({ method: 'POST', body: JSON.stringify({ version: 2 }) });
	});

	it('says so when the version could not be published again', async () => {
		apiFetch.mockResolvedValue(textResponse('this version is already published', 409));
		const outcome = (await settle(() => post({ version: 2 }))) as { thrown?: { status?: number; body?: { message?: string } } };
		expect(outcome.thrown?.status).toBe(409);
		expect(outcome.thrown?.body?.message).toBe('That version could not be published again. Try again.');
	});
});
