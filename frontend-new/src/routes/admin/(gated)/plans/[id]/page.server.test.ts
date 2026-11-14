import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const plan = { id: 'p1', slug: 'pro', name: 'Pro', subscribers: 3 };
const form = {
	name: ' Pro ',
	slug: 'pro',
	description: 'For a busy warung',
	monthly_tokens: '5 000 000',
	price_label: 'Rp 99.000 / month',
	features: 'Everything in Free\n\nPriority answers\n',
	card_tone: 'ember',
	promo_label: '',
	promo_price_label: '',
	promo_ends_at: '',
	is_active: 'on',
	sort_order: '2',
};

function event(id: string, options: { form?: Record<string, string>; answer?: (call: BackendCall) => Response } = {}) {
	const api = fakeBackend(options.answer ?? (() => json([plan])));
	return { api, event: fakeEvent({ url: `http://localhost/admin/plans/${id}`, params: { id }, form: options.form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('a plan', () => {
	it('opens a new plan empty, and an existing one from the list (there is no single-plan endpoint)', async () => {
		expect(await load(event('new').event)).toMatchObject({ plan: null });
		const existing = event('p1');
		expect(await load(existing.event)).toMatchObject({ title: 'Pro', plan: { ok: true, data: plan } });
		expect(existing.api.calls[0].url).toBe('http://api.test/api/admin/plans');
		await expect(load(event('nope').event)).rejects.toMatchObject({ status: 404 });
	});

	it('creates a plan from the form, features one per line', async () => {
		const { api, event: saving } = event('new', { form, answer: () => json({ id: 'p9' }, 201) });
		expect(await thrown(() => actions.save(saving))).toMatchObject({ status: 303, location: '/admin/plans/p9' });
		expect(api.calls[0]).toMatchObject({
			method: 'POST',
			url: 'http://api.test/api/admin/plans',
			body: {
				slug: 'pro',
				name: 'Pro',
				description: 'For a busy warung',
				monthly_tokens: 5_000_000,
				price_label: 'Rp 99.000 / month',
				features: ['Everything in Free', 'Priority answers'],
				card_tone: 'ember',
				promo_label: null,
				promo_price_label: null,
				promo_ends_at: null,
				is_active: true,
				sort_order: 2,
			},
		});
	});

	it('saves an existing plan, hidden and with a promotion until a day', async () => {
		const { api, event: saving } = event('p1', { form: { ...form, is_active: '', promo_label: 'Ramadan', promo_price_label: 'Rp 49.000', promo_ends_at: '2026-12-31' }, answer: () => json(plan) });
		expect(await actions.save(saving)).toMatchObject({ action: 'save', ok: true });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/plans/p1' });
		expect(api.calls[0].body).toMatchObject({ is_active: false, promo_label: 'Ramadan', promo_price_label: 'Rp 49.000', promo_ends_at: '2026-12-31T23:59:59.000Z' });
	});

	it('refuses what the backend would, before sending', async () => {
		for (const wrong of [{ name: '' }, { slug: 'Pro Plan' }, { monthly_tokens: '0' }, { monthly_tokens: 'many' }, { card_tone: 'neon' }, { promo_ends_at: 'soon' }, { sort_order: 'first' }]) {
			const { api, event: saving } = event('p1', { form: { ...form, ...wrong } });
			expect(await actions.save(saving)).toMatchObject({ status: 400, data: { action: 'save', ok: false } });
			expect(api.calls).toEqual([]);
		}
	});

	it('deletes a plan and goes back to the list, or says why it can’t', async () => {
		const gone = event('p1', { answer: () => new Response(null, { status: 204 }) });
		expect(await thrown(() => actions.delete(gone.event))).toMatchObject({ status: 303, location: '/admin/plans' });
		expect(gone.api.calls[0]).toMatchObject({ method: 'DELETE', url: 'http://api.test/api/admin/plans/p1' });

		const used = event('p1', { answer: () => json({ error: 'people are or were on this plan; hide it instead' }, 409) });
		expect(await actions.delete(used.event)).toMatchObject({ status: 409, data: { problem: { message: 'people are or were on this plan; hide it instead' } } });
	});
});
