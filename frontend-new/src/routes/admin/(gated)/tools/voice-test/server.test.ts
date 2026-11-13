import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { POST } = await import('./+server');

const call = (reply: Response | 'unreachable', body: unknown = { text: 'Halo' }) => {
	const api = fakeBackend(() => reply);
	const base = fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } });
	const request = new Request('http://localhost/admin/tools/voice-test', { method: 'POST', body: JSON.stringify(body) });
	return { api, run: () => POST({ ...base, request } as never) };
};

describe('the voice test', () => {
	it('says the line with the saved voice and hands back the recording', async () => {
		const { api, run } = call(new Response(new Uint8Array([1, 2, 3]), { headers: { 'content-type': 'audio/mpeg' } }));
		const response = await run();
		expect(response.headers.get('content-type')).toBe('audio/mpeg');
		expect([...new Uint8Array(await response.arrayBuffer())]).toEqual([1, 2, 3]);
		expect(api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/tools/speak/test', body: { text: 'Halo' } });
	});

	it('passes on what the voice service said when it refuses', async () => {
		const response = await call(json({ error: 'voice service rejected the key' }, 400)).run();
		expect(response.status).toBe(400);
		expect(await response.json()).toEqual({ error: 'voice service rejected the key' });
	});
});
