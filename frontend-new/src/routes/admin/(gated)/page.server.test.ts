import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

const dashboard = { total_users: 3, tokens_today: 150, tokens_all_time: 425, running_agents: 1, sandbox: { running: 2, starting: 3, error: 1, minutes_this_month: 90 } };
const run = (answer: Parameters<typeof fakeBackend>[0]) => {
	const api = fakeBackend(answer);
	return { api, result: () => load(fakeEvent({ url: 'http://localhost/admin', fetch: api.fetch, locals: { accessToken: 't' } }) as never) };
};

describe('the overview', () => {
	it('shows the dashboard’s numbers, sandboxes included', async () => {
		const { api, result } = run(() => json(dashboard));
		expect(await result()).toEqual({ dashboard: { ok: true, data: dashboard } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/dashboard');
	});

	it('says when the numbers couldn’t be read, with the backend’s reason, instead of showing zeros', async () => {
		const { result } = run(() => new Response('failed to count sandboxes', { status: 500 }));
		expect(await result()).toEqual({ dashboard: { ok: false, problem: { status: 500, message: 'failed to count sandboxes' } } });
	});

	it('shows the no-access page when the backend refuses the area', async () => {
		const { result } = run(() => new Response("you don't have access to this part of the admin console", { status: 403 }));
		expect(await thrown(result)).toMatchObject({ status: 403 });
	});
});
