import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json } from './testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { joinWaitlist, loadPayment, loadPlan, startCheckout } = await import('./plan');

const eventWith = (answer: Parameters<typeof fakeBackend>[0]) => {
	const backend = fakeBackend(answer);
	return { backend, event: fakeEvent({ fetch: backend.fetch, locals: { accessToken: 'token' } }) };
};

describe('Plan & usage on the server', () => {
	it('loads the month asked for, the plans, the person’s own keys and when they joined', async () => {
		const { backend, event } = eventWith((call) => {
			if (call.url.includes('/api/usage')) return json({ month: '2026-09' });
			if (call.url.endsWith('/api/plans')) return json({ plans: [], current_plan_id: 'p1' });
			if (call.url.endsWith('/api/llm/connections')) return json({ connections: [{ id: 'k1' }] });
			return json({ joined_at: '2026-07-14T00:00:00Z' });
		});
		expect(await loadPlan(event, '2026-09')).toEqual({ usage: { month: '2026-09' }, plans: { plans: [], current_plan_id: 'p1' }, keys: [{ id: 'k1' }], joinedAt: '2026-07-14T00:00:00Z' });
		expect(backend.calls.map((call) => call.url)).toContain('http://api.test/api/usage?month=2026-09');
	});

	it('asks for this month when the month asked for isn’t one', async () => {
		const { backend, event } = eventWith(() => json({}));
		await loadPlan(event, 'october');
		expect(backend.calls.map((call) => call.url)).toContain('http://api.test/api/usage');
	});

	it('leaves out what fails beside the usage, and says it couldn’t load without the usage', async () => {
		const partial = eventWith((call) => (call.url.includes('/api/usage') ? json({ month: '2026-10' }) : new Response('', { status: 500 })));
		expect(await loadPlan(partial.event, null)).toEqual({ usage: { month: '2026-10' }, plans: null, keys: null, joinedAt: null });
		const down = eventWith(() => 'unreachable');
		expect(await loadPlan(down.event, null)).toBeNull();
	});

	it('joins a plan’s waitlist, and starts paying for one on the gateway’s page', async () => {
		const { backend, event } = eventWith((call) => (call.url.endsWith('/waitlist') ? json({ on_waitlist: true }) : json({ url: 'https://checkout.xendit.co/web/abc' })));
		expect(await joinWaitlist(event, 'pro 1')).toBe(true);
		expect(await startCheckout(event, 'pro 1')).toEqual({ url: 'https://checkout.xendit.co/web/abc' });
		expect(backend.calls.map((call) => [call.method, call.url.replace('http://api.test', '')])).toEqual([
			['POST', '/api/plans/pro%201/waitlist'],
			['POST', '/api/plans/pro%201/checkout'],
		]);
	});

	it('passes the backend’s reason on when paying can’t start, and never follows a page that isn’t https', async () => {
		expect(await startCheckout(eventWith(() => json({ error: 'payments_not_configured' }, 503)).event, 'pro')).toEqual({ error: 'payments_not_configured', status: 503 });
		expect(await startCheckout(eventWith(() => json({ error: 'already_on_plan' }, 409)).event, 'pro')).toEqual({ error: 'already_on_plan', status: 409 });
		expect(await startCheckout(eventWith(() => json({ url: 'javascript:alert(1)' })).event, 'pro')).toEqual({ error: 'gateway_error', status: 502 });
		expect(await joinWaitlist(eventWith(() => new Response('', { status: 404 })).event, 'gone')).toBe(false);
	});

	it('reads the payment the person came back from', async () => {
		const paid = { id: 'pay1', status: 'paid', plan_name: 'Pro', amount: 99000, currency: 'IDR', invoice_url: null, period_end: '2026-12-15T00:00:00Z' };
		const { backend, event } = eventWith(() => json(paid));
		expect(await loadPayment(event, 'pay1')).toEqual(paid);
		expect(backend.calls[0].url).toBe('http://api.test/api/billing/payments/pay1');
		expect(await loadPayment(eventWith(() => new Response('', { status: 404 })).event, 'someone-elses')).toBeNull();
		expect(await loadPayment(eventWith(() => json(paid)).event, '../users')).toBeNull();
	});
});
