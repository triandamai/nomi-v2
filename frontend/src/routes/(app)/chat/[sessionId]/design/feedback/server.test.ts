import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, json, settle, textResponse } from '../../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { POST } = await import('./+server');

const draft = { version: 2, level: 'global', body: 'Warmer' };
const post = (body: unknown = draft) =>
	POST({ params: { sessionId: 's1' }, request: new Request('http://localhost/chat/s1/design/feedback', { method: 'POST', body: JSON.stringify(body) }), cookies: cookieJar(), fetch } as never) as Promise<Response>;

beforeEach(() => apiFetch.mockReset());

describe('design studio: feedback', () => {
	it('passes feedback on to the chat’s Studio as the person', async () => {
		apiFetch.mockResolvedValue(json({ number: 3, ...draft }, 201));
		const response = await post();
		expect(response.status).toBe(201);
		expect(await response.json()).toMatchObject({ number: 3, body: 'Warmer' });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/design/feedback');
		expect(apiFetch.mock.calls[0][3]).toMatchObject({ method: 'POST', body: JSON.stringify(draft) });
	});

	it('says why feedback was refused', async () => {
		apiFetch.mockResolvedValue(textResponse('this version has no page "about.html"', 422));
		const outcome = (await settle(() => post())) as { thrown?: { status?: number; body?: { message?: string } } };
		expect(outcome.thrown?.status).toBe(422);
		expect(outcome.thrown?.body?.message).toBe('this version has no page "about.html"');
	});

	it('falls back to its own words when the refusal has none', async () => {
		apiFetch.mockResolvedValue(textResponse('', 500));
		const outcome = (await settle(() => post())) as { thrown?: { status?: number; body?: { message?: string } } };
		expect(outcome.thrown?.body?.message).toBe('That feedback could not be saved. Try again.');
	});
});
