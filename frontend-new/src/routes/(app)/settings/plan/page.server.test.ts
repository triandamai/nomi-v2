import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

const form = (body: Record<string, string>, answer: Parameters<typeof fakeBackend>[0]) => {
	const api = fakeBackend(answer);
	return { api, event: fakeEvent({ url: 'http://localhost/settings/plan', fetch: api.fetch, locals: { accessToken: 'token' }, form: body }) };
};

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

	it('shows the payment the person came back from, and nothing for an address without one', async () => {
		const api = fakeBackend((call) => json(call.url.includes('/api/usage') ? { month: '2026-11' } : call.url.includes('/billing/payments/') ? { id: 'pay1', status: 'paid', plan_name: 'Pro' } : {}));
		const data = await load(fakeEvent({ url: 'http://localhost/settings/plan?payment=pay1', fetch: api.fetch, locals: { accessToken: 'token' } }) as never);
		expect(data).toMatchObject({ payment: { id: 'pay1', status: 'paid', plan_name: 'Pro' } });
		const none = await load(fakeEvent({ url: 'http://localhost/settings/plan', fetch: fakeBackend((call) => json(call.url.includes('/api/usage') ? { month: '2026-11' } : {})).fetch, locals: { accessToken: 'token' } }) as never);
		expect(none).toMatchObject({ payment: null });
	});

	it('joins a waitlist, and sends the person to the gateway’s page to pay', async () => {
		const waiting = form({ plan: 'pro' }, () => json({ on_waitlist: true }));
		expect(await actions.waitlist(waiting.event as never)).toEqual({ waitlisted: 'pro' });
		const paying = form({ plan: 'pro' }, () => json({ url: 'https://checkout.xendit.co/web/abc' }));
		expect(await thrown(() => actions.checkout(paying.event as never))).toMatchObject({ status: 303, location: 'https://checkout.xendit.co/web/abc' });
	});

	it('says why paying couldn’t start, and asks for a plan', async () => {
		expect(await actions.checkout(form({ plan: 'pro' }, () => json({ error: 'payments_not_configured' }, 503)).event as never)).toMatchObject({ status: 503, data: { message: 'Payments aren’t set up on this Nomi yet.' } });
		expect(await actions.checkout(form({ plan: 'pro' }, () => json({ error: 'already_on_plan' }, 409)).event as never)).toMatchObject({ status: 409, data: { message: 'You’re on this plan already.' } });
		expect(await actions.checkout(form({}, () => json({})).event as never)).toMatchObject({ status: 400 });
		expect(await actions.waitlist(form({ plan: 'gone' }, () => new Response('', { status: 404 })).event as never)).toMatchObject({ status: 502 });
	});
});
