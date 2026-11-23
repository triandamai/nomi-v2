import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load } = await import('./+page.server');

const PROJECT = '1b4e28ba-2fa1-11d2-883f-0016d3cca427';

describe('admin sandboxes', () => {
	it('shows one project’s sandboxes when asked', async () => {
		const api = fakeBackend(() => json({ sandboxes: [] }));
		expect(await load(fakeEvent({ url: `http://localhost/admin/sandboxes?project=${PROJECT}`, fetch: api.fetch, locals: { accessToken: 't' } }) as never)).toEqual({ project: PROJECT, sandboxes: { ok: true, data: [] } });
		expect(api.calls[0].url).toBe(`http://api.test/api/admin/sandboxes?project=${PROJECT}`);
	});

	it('ignores a project that isn’t an id', async () => {
		const api = fakeBackend(() => json({ sandboxes: [] }));
		await load(fakeEvent({ url: 'http://localhost/admin/sandboxes?project=../x', fetch: api.fetch, locals: { accessToken: 't' } }) as never);
		expect(api.calls[0].url).toBe('http://api.test/api/admin/sandboxes');
	});
});
