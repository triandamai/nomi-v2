import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const saved = { enabled: false, message: '', until: null, updated_at: '2026-11-11T05:00:00Z' };

describe('maintenance', () => {
	it('reads whether Nomi is down for maintenance', async () => {
		const api = fakeBackend(() => json(saved));
		expect(await load(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toEqual({ maintenance: { ok: true, data: saved } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/maintenance');
	});

	it('turns it on with a message and when Nomi is back (UTC), or off', async () => {
		const on = fakeBackend(() => json(saved));
		const result = await actions.save(fakeEvent({ form: { enabled: 'on', message: ' Moving to a bigger database. ', until_date: '2026-11-12', until_time: '03:30' }, fetch: on.fetch, locals: { accessToken: 't' } }) as never);
		expect(result).toMatchObject({ action: 'save', ok: true });
		expect(on.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/maintenance', body: { enabled: true, message: 'Moving to a bigger database.', until: '2026-11-12T03:30:00.000Z' } });

		const off = fakeBackend(() => json(saved));
		await actions.save(fakeEvent({ form: { enabled: '', message: '', until_date: '', until_time: '' }, fetch: off.fetch, locals: { accessToken: 't' } }) as never);
		expect(off.calls[0].body).toEqual({ enabled: false, message: '', until: null });
	});

	it('refuses a time it can’t read, and a message over 500 characters', async () => {
		for (const form of [
			{ enabled: 'on', message: '', until_date: '12/11/2026', until_time: '' },
			{ enabled: 'on', message: '', until_date: '2026-11-12', until_time: '25:00' },
			{ enabled: 'on', message: 'x'.repeat(501), until_date: '', until_time: '' },
		]) {
			const api = fakeBackend(() => json(saved));
			expect(await actions.save(fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toMatchObject({ status: 400, data: { action: 'save' } });
			expect(api.calls).toEqual([]);
		}
	});
});
