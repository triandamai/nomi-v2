import { describe, expect, it, vi } from 'vitest';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { getMaintenance } = await import('./maintenance');

const respond = (response: Response | Error) =>
	vi.fn(async () => {
		if (response instanceof Error) throw response;
		return response;
	}) as unknown as typeof fetch;

describe('getMaintenance', () => {
	it('says when Nomi is down for maintenance, and until when', async () => {
		const fetchFn = respond(Response.json({ enabled: true, message: 'Upgrading the database', until: '2026-11-10T10:00:00Z', updated_at: '2026-11-10T09:00:00Z' }));
		expect(await getMaintenance(fetchFn)).toEqual({ message: 'Upgrading the database', until: '2026-11-10T10:00:00Z' });
	});

	it('lets people in when it is off, or when the check itself fails', async () => {
		expect(await getMaintenance(respond(Response.json({ enabled: false, message: null, until: null, updated_at: null })))).toBeNull();
		expect(await getMaintenance(respond(new Response(null, { status: 500 })))).toBeNull();
		expect(await getMaintenance(respond(new Error('backend down')))).toBeNull();
	});
});
