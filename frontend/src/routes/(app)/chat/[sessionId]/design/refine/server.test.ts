import { beforeEach, describe, expect, it, vi } from 'vitest';
import { cookieJar, json, settle, textResponse } from '../../../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { POST } = await import('./+server');

const post = (body: unknown) =>
	POST({ params: { sessionId: 's1' }, request: new Request('http://localhost/chat/s1/design/refine', { method: 'POST', body: JSON.stringify(body) }), cookies: cookieJar(), fetch } as never) as Promise<Response>;

beforeEach(() => apiFetch.mockReset());

describe('design studio: ask Sora for a new version', () => {
	it('sends the request to the chat as a message from the person', async () => {
		apiFetch.mockResolvedValue(json({ user_message: { id: 'm1' } }));
		const response = await post({ text: '@Sora Please act on feedback #2.' });
		expect(response.status).toBe(202);
		expect(apiFetch.mock.calls[0][2]).toBe('/api/sessions/s1/messages');
		expect(apiFetch.mock.calls[0][3]).toMatchObject({ method: 'POST', body: JSON.stringify({ text: '@Sora Please act on feedback #2.' }) });
	});

	it('sends nothing without words', async () => {
		for (const body of [{}, { text: '  ' }, { text: 7 }]) {
			const outcome = (await settle(() => post(body))) as { thrown?: { status?: number } };
			expect(outcome.thrown?.status).toBe(400);
		}
		expect(apiFetch).not.toHaveBeenCalled();
	});

	it('says so when the chat did not take the message', async () => {
		apiFetch.mockResolvedValue(textResponse('busy', 409));
		const outcome = (await settle(() => post({ text: '@Sora go' }))) as { thrown?: { status?: number; body?: { message?: string } } };
		expect(outcome.thrown?.status).toBe(409);
		expect(outcome.thrown?.body?.message).toBe('That could not be sent to Sora. Try again.');
	});
});
