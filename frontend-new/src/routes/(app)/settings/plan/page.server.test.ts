import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

describe('Settings › Plan & usage', () => {
	it('names itself and loads the month in the address', async () => {
		const api = fakeBackend((call) => json(call.url.includes('/api/usage') ? { month: '2026-09' } : {}));
		const data = await load(fakeEvent({ url: 'http://localhost/settings/plan?month=2026-09', fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ title: 'Plan & usage', csv: '/settings/plan/usage.csv?month=2026-09', usage: { month: '2026-09' } });
	});

	it('shows the error page without the usage', async () => {
		const api = fakeBackend(() => 'unreachable');
		expect(await thrown(() => load(fakeEvent({ url: 'http://localhost/settings/plan', fetch: api.fetch, locals: { accessToken: 'token' } }) as never))).toMatchObject({ status: 503 });
	});
});
