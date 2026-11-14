import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

const run = async (url: string, answer = json({ users: [], total: 0 })) => {
	const api = fakeBackend(() => answer.clone());
	const result = await load(fakeEvent({ url, fetch: api.fetch, locals: { accessToken: 't' } }) as never);
	return { calls: api.calls.map((call) => call.url), result };
};

describe('users', () => {
	it('asks for the page and search in the address, twenty at a time', async () => {
		expect((await run('http://localhost/admin/users')).calls).toEqual(['http://api.test/api/admin/users?page=1&page_size=20']);
		expect((await run('http://localhost/admin/users?query=%20ana%20&page=3')).calls).toEqual(['http://api.test/api/admin/users?page=3&page_size=20&query=ana']);
		expect((await run('http://localhost/admin/users?page=-2')).calls).toEqual(['http://api.test/api/admin/users?page=1&page_size=20']);
	});

	it('hands the page what it searched, and the list', async () => {
		const { result } = await run('http://localhost/admin/users?query=ana&page=2', json({ users: [{ id: 'u1' }], total: 21 }));
		expect(result).toEqual({ list: { ok: true, data: { users: [{ id: 'u1' }], total: 21 } }, query: 'ana', page: 2, pageSize: 20 });
	});

	it('shows the no-access page when the backend refuses the area', async () => {
		await expect(run('http://localhost/admin/users', json({ error: 'forbidden' }, 403))).rejects.toMatchObject({ status: 403 });
	});
});
