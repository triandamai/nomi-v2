import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const route = await import('./+server');

const usage = { month: '2026-10', current_month: '2026-11', calls: 3 };

describe('the Tokens & cost sheet’s month', () => {
	it('asks the backend for the month asked for', async () => {
		const api = fakeBackend(() => json(usage));
		const response = await route.GET(fakeEvent({ url: 'http://localhost/projects/p1/usage?month=2026-10', params: { id: 'p1' }, locals: { accessToken: 't' }, fetch: api.fetch }) as never);
		expect(api.calls[0].url).toBe('http://api.test/api/projects/p1/usage?month=2026-10');
		expect(await response.json()).toEqual(usage);
	});

	it('asks for this month when no month, or a malformed one, is given', async () => {
		const api = fakeBackend(() => json(usage));
		await route.GET(fakeEvent({ url: 'http://localhost/projects/p1/usage?month=2026-1x', params: { id: 'p1' }, locals: { accessToken: 't' }, fetch: api.fetch }) as never);
		expect(api.calls[0].url).toBe('http://api.test/api/projects/p1/usage');
	});

	it('passes a refusal on, so the sheet can say it couldn’t load', async () => {
		const api = fakeBackend(() => new Response('project not found', { status: 404 }));
		const response = await route.GET(fakeEvent({ url: 'http://localhost/projects/p1/usage', params: { id: 'p1' }, locals: { accessToken: 't' }, fetch: api.fetch }) as never);
		expect(response.status).toBe(404);
	});
});
