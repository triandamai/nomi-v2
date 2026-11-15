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

	it('reads and saves the memory settings and the profile', async () => {
		const settings = { learn_from_chats: true, keep_profile: true, search_past_chats: true, memory_budget: 8000, profile_budget: 1000, memory_used: 120, profile_used: 30 };
		const { backend, event } = eventWith((call) => (call.url.endsWith('/settings') ? json(settings) : json({ text: 'Mario.', tokens: 2, updated_at: null })));
		expect(await memory.loadMemorySettings(event)).toEqual(settings);
		expect(await memory.loadProfile(event)).toEqual({ text: 'Mario.', tokens: 2, updated_at: null });
		expect(await memory.saveMemorySettings(event, { search_past_chats: false })).toEqual({ ok: true, data: settings });
		expect(await memory.saveProfile(event, 'Mario.')).toMatchObject({ ok: true });
		expect(backend.calls.slice(2).map((call) => [call.method, call.url, call.body])).toEqual([
			['PUT', 'http://api.test/api/memory/settings', { search_past_chats: false }],
			['PUT', 'http://api.test/api/memory/profile', { text: 'Mario.' }],
		]);
	});

	it('passes on the backend’s refusal: a budget out of range, a profile too long', async () => {
		const { event } = eventWith((call) => json({ error: call.url.endsWith('/settings') ? 'invalid_budget' : 'too_long' }, 400));
		expect(await memory.saveMemorySettings(event, { memory_budget: 10 })).toMatchObject({ ok: false, status: 400, code: 'invalid_budget' });
		expect(await memory.saveProfile(event, 'x')).toMatchObject({ ok: false, status: 400, code: 'too_long' });
	});
});

