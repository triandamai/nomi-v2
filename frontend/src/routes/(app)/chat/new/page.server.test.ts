import { beforeEach, describe, expect, it, vi } from 'vitest';
import { isRedirect, type Cookies } from '@sveltejs/kit';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions } = await import('./+page.server');

const json = (body: unknown, status = 200) => new Response(JSON.stringify(body), { status, headers: { 'content-type': 'application/json' } });

function send(fields: Record<string, string>) {
	const body = new FormData();
	for (const [name, value] of Object.entries(fields)) body.set(name, value);
	const request = new Request('http://localhost/chat/new?/sendMessage', { method: 'POST', body });
	const event = { request, cookies: {} as Cookies, fetch } as unknown as Parameters<NonNullable<typeof actions.sendMessage>>[0];
	return Promise.resolve()
		.then(() => actions.sendMessage!(event))
		.catch((e: unknown) => e);
}

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async (_fetch: unknown, _cookies: unknown, path: string, init?: RequestInit) => {
		if (path === '/api/sessions' && init?.method === 'POST') return json({ session_id: 's1' }, 201);
		if (path === '/api/sessions/s1/model') return json({ model: { key: 'admin:a2' } });
		return json({});
	});
});

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => `${(init as RequestInit | undefined)?.method ?? 'GET'} ${path}`);

describe('new chat: sendMessage', () => {
	it('sets the picked model on the new chat before the first message', async () => {
		const outcome = await send({ text: 'hi', model: 'conn:c1:openai/gpt-5' });
		expect(isRedirect(outcome) && outcome.location).toBe('/chat/s1');
		expect(calls()).toEqual(['POST /api/sessions', 'PUT /api/sessions/s1/model', 'POST /api/sessions/s1/messages']);
		const put = apiFetch.mock.calls[1][3] as RequestInit;
		expect(JSON.parse(String(put.body))).toEqual({ key: 'conn:c1:openai/gpt-5' });
	});

	it('leaves the default model alone when none was picked', async () => {
		await send({ text: 'hi' });
		expect(calls()).toEqual(['POST /api/sessions', 'POST /api/sessions/s1/messages']);
	});

	it("still sends the message when the model can't be set", async () => {
		apiFetch.mockImplementation(async (_fetch: unknown, _cookies: unknown, path: string, init?: RequestInit) => {
			if (path === '/api/sessions' && init?.method === 'POST') return json({ session_id: 's1' }, 201);
			if (path === '/api/sessions/s1/model') return json({ error: 'forbidden' }, 403);
			return json({});
		});
		const outcome = await send({ text: 'hi', model: 'admin:a2' });
		expect(isRedirect(outcome) && outcome.location).toBe('/chat/s1');
		expect(calls()).toContain('POST /api/sessions/s1/messages');
	});
});
