import { describe, expect, it, vi } from 'vitest';
import { startChat } from './start';

const answer = (status: number, body: unknown) => vi.fn(async () => Response.json(body, { status })) as unknown as typeof fetch;

describe('starting a chat', () => {
	it('opens the new chat, on the model picked for it', async () => {
		const fetchFn = answer(200, { sessionId: 's1', sent: true });
		expect(await startChat('Halo', 'deepseek-flash', fetchFn)).toEqual({ kind: 'opened', href: '/chat/s1' });
		expect(fetchFn).toHaveBeenCalledWith('/chat', expect.objectContaining({ method: 'POST', body: '{"text":"Halo","model":"deepseek-flash"}' }));
	});

	it('sends no model when the person’s default is used', async () => {
		const fetchFn = answer(200, { sessionId: 's1', sent: false });
		expect(await startChat('Halo', null, fetchFn)).toEqual({ kind: 'opened', href: '/chat/s1' });
		expect(fetchFn).toHaveBeenCalledWith('/chat', expect.objectContaining({ body: '{"text":"Halo"}' }));
	});

	it('tells offline, a sign-in that ran out, and a refusal apart', async () => {
		const offline = vi.fn(async () => {
			throw new TypeError('Failed to fetch');
		}) as unknown as typeof fetch;
		expect(await startChat('Halo', null, offline)).toEqual({ kind: 'offline' });
		expect(await startChat('Halo', null, answer(401, { error: 'signed_out' }))).toEqual({ kind: 'signed_out' });
		expect(await startChat('Halo', null, answer(503, { error: 'backend unavailable' }))).toEqual({ kind: 'failed', message: 'backend unavailable' });
	});
});
