import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const memory = await import('./memory');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('Memory on the server', () => {
	it('loads the memories without their vectors', async () => {
		const { event } = eventWith(() => json({ memories: [{ id: 'm1', content: 'No emoji', embedding: [0.1, 0.2] }] }));
		expect(await memory.loadMemories(event)).toEqual([{ id: 'm1', content: 'No emoji' }]);
	});

	it('says it couldn’t load', async () => {
		const { event } = eventWith(() => 'unreachable');
		expect(await memory.loadMemories(event)).toBeNull();
	});

	it('corrects, confirms and forgets a memory', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		await memory.editMemory(event, 'm1', 'Short answers');
		await memory.confirmMemory(event, 'm1');
		await memory.forgetMemory(event, 'm1');
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body])).toEqual([
			['PUT', '/api/memory/m1', { content: 'Short answers' }],
			['POST', '/api/memory/m1/confirm', undefined],
			['DELETE', '/api/memory/m1', undefined],
		]);
	});
});
