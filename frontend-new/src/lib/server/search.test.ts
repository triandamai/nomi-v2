import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { searchAll } = await import('./search');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('search on the server', () => {
	it('asks the backend for chats, memory and files matching what was typed', async () => {
		const hit = { kind: 'chat', id: 's1', title: 'Budget', snippet: null, session_id: 's1', updated_at: '2026-11-15T09:00:00Z' };
		const { backend, event } = eventWith(() => json({ results: [hit] }));
		expect(await searchAll(event, '  belanja & kopi ')).toEqual([hit]);
		expect(backend.calls[0].url).toBe('http://api.test/api/search?q=belanja+%26+kopi&kinds=chat%2Cmemory%2Cfile&limit=20');
	});

	it('asks nothing for nothing typed, and says when it couldn’t look', async () => {
		const { backend, event } = eventWith(() => json({ results: [] }));
		expect(await searchAll(event, '  ')).toEqual([]);
		expect(backend.calls).toEqual([]);
		expect(await searchAll(eventWith(() => new Response('', { status: 500 })).event, 'kopi')).toBeNull();
	});
});
