import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

const run = async (url: string) => {
	const api = fakeBackend(() => json({ month: '2026-10' }));
	const result = await load(fakeEvent({ url, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
	return { calls: api.calls.map((call) => call.url), result };
};

describe('usage', () => {
	it('asks for the month in the address, and this month when there is none or it isn’t one', async () => {
		expect((await run('http://localhost/admin/usage?month=2026-10')).calls).toEqual(['http://api.test/api/admin/usage?month=2026-10']);
		expect((await run('http://localhost/admin/usage')).calls).toEqual(['http://api.test/api/admin/usage']);
		expect((await run('http://localhost/admin/usage?month=oct')).calls).toEqual(['http://api.test/api/admin/usage']);
		expect((await run('http://localhost/admin/usage')).result).toEqual({ usage: { ok: true, data: { month: '2026-10' } } });
	});
});
