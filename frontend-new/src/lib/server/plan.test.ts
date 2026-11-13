import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { loadPlan } = await import('./plan');

const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('Plan & usage on the server', () => {
	it('loads the month asked for, the plans, the person’s own keys and when they joined', async () => {
		const { backend, event } = eventWith((call) => {
			if (call.url.includes('/api/usage')) return json({ month: '2026-09' });
			if (call.url.endsWith('/api/plans')) return json({ plans: [], current_plan_id: 'p1' });
			if (call.url.endsWith('/api/llm/connections')) return json({ connections: [{ id: 'k1' }] });
			return json({ joined_at: '2026-07-14T00:00:00Z' });
		});
		expect(await loadPlan(event, '2026-09')).toEqual({ usage: { month: '2026-09' }, plans: { plans: [], current_plan_id: 'p1' }, keys: [{ id: 'k1' }], joinedAt: '2026-07-14T00:00:00Z' });
		expect(backend.calls.map((call) => call.url)).toContain('http://api.test/api/usage?month=2026-09');
	});

	it('asks for this month when the month asked for isn’t one', async () => {
		const { backend, event } = eventWith(() => json({}));
		await loadPlan(event, 'october');
		expect(backend.calls.map((call) => call.url)).toContain('http://api.test/api/usage');
	});

	it('leaves out what fails beside the usage, and says it couldn’t load without the usage', async () => {
		const partial = eventWith((call) => (call.url.includes('/api/usage') ? json({ month: '2026-10' }) : new Response('', { status: 500 })));
		expect(await loadPlan(partial.event, null)).toEqual({ usage: { month: '2026-10' }, plans: null, keys: null, joinedAt: null });
		const down = eventWith(() => 'unreachable');
		expect(await loadPlan(down.event, null)).toBeNull();
	});
});
