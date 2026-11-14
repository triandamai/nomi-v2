import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const article = { id: 'k1', title: 'Ongkir', body: '…', tags: [], published: true };
function event(id: string, fields?: Record<string, string>, reply: (call: BackendCall) => Response = () => json([article])) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ params: { id }, form: fields, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('an article', () => {
	it('opens new or from the list', async () => {
		expect(await load(event('new').event)).toMatchObject({ article: null });
		expect(await load(event('k1').event)).toMatchObject({ title: 'Ongkir', article: { ok: true, data: article } });
		await expect(load(event('nope').event)).rejects.toMatchObject({ status: 404 });
	});

	it('is written, as a draft or published, and goes back to support', async () => {
		const fields = { title: 'Retur', body: 'Tujuh hari.', tags: 'retur', published: '' };
		const created = event('new', fields, () => json({ id: 'k2' }, 201));
		expect(await thrown(() => actions.save(created.event))).toMatchObject({ status: 303, location: '/admin/support' });
		expect(created.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/support/articles', body: { title: 'Retur', body: 'Tujuh hari.', tags: ['retur'], published: false } });

		const saved = event('k1', { ...fields, published: 'on' }, () => json(article));
		expect(await actions.save(saved.event)).toMatchObject({ action: 'save', ok: true });
		expect(saved.api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/support/articles/k1', body: { published: true } });

		const untitled = event('k1', { ...fields, title: '' });
		expect(await actions.save(untitled.event)).toMatchObject({ status: 400 });
	});

	it('is deleted', async () => {
		const deleted = event('k1', {}, () => new Response(null, { status: 204 }));
		expect(await thrown(() => actions.delete(deleted.event))).toMatchObject({ status: 303, location: '/admin/support' });
		expect(deleted.api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/admin/support/articles/k1' });
	});
});
