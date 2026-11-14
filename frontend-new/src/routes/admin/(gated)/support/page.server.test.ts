import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(options: { url?: string; form?: Record<string, string>; reply?: (call: BackendCall) => Response } = {}) {
	const api = fakeBackend(options.reply ?? (() => json([])));
	return { api, event: fakeEvent({ url: options.url ?? 'http://localhost/admin/support', form: options.form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('support', () => {
	it('loads the open drafts, the questions asked, the keys, and the articles searched for', async () => {
		const { api, event: loading } = event({ url: 'http://localhost/admin/support?q=ongkir' });
		expect(await load(loading)).toMatchObject({ query: 'ongkir', drafts: { ok: true }, questions: { ok: true }, keys: { ok: true }, articles: { ok: true } });
		expect(api.calls.map((call) => call.url).sort()).toEqual([
			'http://api.test/api/admin/support/articles?q=ongkir',
			'http://api.test/api/admin/support/drafts?status=open',
			'http://api.test/api/admin/support/keys',
			'http://api.test/api/admin/support/questions',
		]);
	});

	it('publishes a draft with the edits made to it, or dismisses it', async () => {
		const published = event({ form: { id: 'd1', title: ' Ongkir ', body: 'Gratis di atas Rp 100.000.', tags: 'ongkir, pengiriman ,' }, reply: () => json({ id: 'k1' }) });
		expect(await actions.publishDraft(published.event)).toMatchObject({ action: 'publishDraft', ok: true });
		expect(published.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/support/drafts/d1/publish', body: { title: 'Ongkir', body: 'Gratis di atas Rp 100.000.', tags: ['ongkir', 'pengiriman'], published: true } });

		const dismissed = event({ form: { id: 'd1' }, reply: () => new Response(null, { status: 204 }) });
		expect(await actions.dismissDraft(dismissed.event)).toMatchObject({ action: 'dismissDraft', ok: true });
		expect(dismissed.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/support/drafts/d1/dismiss' });
	});

	it('makes a key and hands its secret back once; revokes one', async () => {
		const made = event({ form: { name: 'Toko Bu Siti', product_name: 'Kasir' }, reply: () => json({ id: 'k1', name: 'Toko Bu Siti', secret: 'nsk_abc' }, 201) });
		expect(await actions.createKey(made.event)).toMatchObject({ action: 'createKey', ok: true, data: { secret: 'nsk_abc' } });
		expect(made.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/support/keys', body: { name: 'Toko Bu Siti', product_name: 'Kasir' } });
		const unnamed = event({ form: { name: '' } });
		expect(await actions.createKey(unnamed.event)).toMatchObject({ status: 400 });

		const revoked = event({ form: { id: 'k1' }, reply: () => new Response(null, { status: 204 }) });
		expect(await actions.revokeKey(revoked.event)).toMatchObject({ action: 'revokeKey', ok: true });
		expect(revoked.api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/admin/support/keys/k1' });
	});
});
