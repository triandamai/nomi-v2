import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

const SESSION = '0f8a2c1e-7b3d-4e5f-9a6b-1c2d3e4f5a6b';
const event = (agentSessionId: string, type = 'ToolCalled') => ({ id: `e-${agentSessionId}-${type}`, session_id: SESSION, agent_session_id: agentSessionId, agent_type: 'money', agent_display_name: 'Money', event_type: type, created_at: '', tool_name: null, is_error: null });

const run = (url: string, answer: Parameters<typeof fakeBackend>[0]) => {
	const api = fakeBackend(answer);
	return { api, result: () => load(fakeEvent({ url, fetch: api.fetch, locals: { accessToken: 't' } }) as never) as Promise<Record<string, unknown>> };
};

describe('live agents', () => {
	it('loads everyone’s running agents and the latest 50 events', async () => {
		const { api, result } = run('http://localhost/admin/agents', (call) => (call.url.includes('agent-events') ? json([event('a1')]) : json({ users: [] })));
		expect(await result()).toEqual({ agents: { ok: true, data: { users: [] } }, events: { ok: true, data: [event('a1')] }, detail: null });
		expect(api.calls.map((call) => call.url)).toEqual(['http://api.test/api/admin/agents', 'http://api.test/api/admin/agent-events?limit=50']);
	});

	it('opens one agent’s steps: its chat’s events, only its own', async () => {
		const { api, result } = run(`http://localhost/admin/agents?agent=a1&session=${SESSION}`, (call) =>
			call.url.includes(`session_id=${SESSION}`) ? json([event('a1'), event('a2'), event('a1', 'AgentSpawned')]) : call.url.includes('agent-events') ? json([]) : json({ users: [] }),
		);
		expect((await result()).detail).toEqual({ agent: 'a1', session: SESSION, events: { ok: true, data: [event('a1'), event('a1', 'AgentSpawned')] } });
		expect(api.calls[2].url).toBe(`http://api.test/api/admin/agent-events?session_id=${SESSION}&limit=200`);
	});

	it('doesn’t ask about a chat that isn’t an id', async () => {
		const { api, result } = run('http://localhost/admin/agents?agent=a1&session=../x', () => json({ users: [] }));
		expect((await result()).detail).toBeNull();
		expect(api.calls).toHaveLength(2);
	});

	it('keeps the page when only the events fail, and shows no access when the area is refused', async () => {
		const failing = run('http://localhost/admin/agents', (call) => (call.url.includes('agent-events') ? new Response('failed to list agent events', { status: 500 }) : json({ users: [] })));
		expect((await failing.result()).events).toEqual({ ok: false, problem: { status: 500, message: 'failed to list agent events' } });
		const refused = run('http://localhost/admin/agents', () => new Response('no', { status: 403 }));
		expect(await thrown(refused.result)).toMatchObject({ status: 403 });
	});
});
