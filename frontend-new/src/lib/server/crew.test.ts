import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const crew = await import('./crew');
const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('Crew on the server', () => {
	it('loads the crew, the model choices and the personality’s versions, each on its own', async () => {
		const { event } = eventWith((call) => {
			if (call.url.endsWith('/api/agents')) return json({ agents: [{ agent_type: 'chitchat', name: 'Nomi' }] });
			if (call.url.endsWith('/api/personality/history')) return json({ versions: [{ version: 2, description: 'Warm', created_at: '2026-11-01T00:00:00Z', is_current: true }] });
			return new Response('', { status: 500 });
		});
		expect(await crew.loadCrew(event)).toEqual({
			members: [{ agent_type: 'chitchat', name: 'Nomi' }],
			options: null,
			models: null,
			personality: [{ version: 2, description: 'Warm', created_at: '2026-11-01T00:00:00Z', is_current: true }],
		});
	});

	it('has no crew when the backend can’t say who’s on it', async () => {
		const { event } = eventWith(() => new Response('', { status: 502 }));
		expect((await crew.loadCrew(event)).members).toBeNull();
	});

	it('restores a personality version, and says when it’s gone', async () => {
		const { backend, event } = eventWith(() => new Response(null, { status: 204 }));
		expect(await crew.restorePersonality(event, 2)).toEqual({ ok: true, data: null });
		expect(backend.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/personality/rollback', body: { version: 2 } });
		const gone = eventWith(() => new Response('version not found', { status: 404 }));
		expect(await crew.restorePersonality(gone.event, 9)).toEqual({ ok: false, status: 404, message: 'version not found' });
	});
});
