import { describe, expect, it, vi } from 'vitest';
import { callJson } from './client';

const answer = (status: number, body: unknown) => vi.fn(async () => Response.json(body, { status })) as unknown as typeof fetch;

describe('callJson', () => {
	it('sends JSON and reads JSON', async () => {
		const fetchFn = answer(200, { sessionId: 'new' });
		expect(await callJson('/chat', { method: 'POST', body: { text: 'hi' } }, fetchFn)).toEqual({ ok: true, data: { sessionId: 'new' } });
		expect(fetchFn).toHaveBeenCalledWith('/chat', { method: 'POST', headers: { 'content-type': 'application/json' }, body: '{"text":"hi"}' });
	});

	it('says what the server refused, and when the sign-in is over', async () => {
		expect(await callJson('/x', {}, answer(409, { error: 'the crew is already working in this chat' }))).toEqual({
			ok: false,
			status: 409,
			message: 'the crew is already working in this chat',
			signedOut: false,
			offline: false,
		});
		expect(await callJson('/x', {}, answer(401, { error: 'signed_out' }))).toMatchObject({ ok: false, signedOut: true });
	});

	it('calls a call that never reached the server offline', async () => {
		const fetchFn = vi.fn(async () => {
			throw new TypeError('Failed to fetch');
		}) as unknown as typeof fetch;
		expect(await callJson('/x', {}, fetchFn)).toEqual({ ok: false, status: 0, message: '', signedOut: false, offline: true });
	});

	it('takes an answer with no body as done', async () => {
		const fetchFn = vi.fn(async () => new Response(null, { status: 204 })) as unknown as typeof fetch;
		expect(await callJson('/x', { method: 'DELETE' }, fetchFn)).toEqual({ ok: true, data: null });
	});
});
