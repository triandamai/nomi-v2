import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, textResponse } from '../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const PAYMENT_ID = '3f2b8c1e-4d5a-4b6c-8d7e-9f0a1b2c3d4e';
const paths = () => apiFetch.mock.calls.map(([, , path]) => path as string);

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async (_f: unknown, _c: unknown, path: string) => {
		if (path === '/api/billing/payments') return json([{ id: PAYMENT_ID }]);
		if (path === '/api/billing/subscription') return json({ auto_renew: true });
		if (path.startsWith('/api/billing/payments/')) return json({ id: PAYMENT_ID, status: 'paid' });
		return json({ ok: true });
	});
});

describe('billing: load', () => {
	it('loads the plan, limits, payments and referral beside the usage', async () => {
		const data = await load(event({ url: 'http://localhost/billing' }) as never);
		expect(data).toMatchObject({ subscription: { auto_renew: true }, payments: [{ id: PAYMENT_ID }], payment: null });
		expect(paths()).toEqual(expect.arrayContaining(['/api/usage', '/api/plans', '/api/billing/subscription', '/api/entitlements', '/api/billing/payments', '/api/billing/referral']));
	});

	it('fetches the payment named in the URL, and only when it is an id', async () => {
		const data = await load(event({ url: `http://localhost/billing?payment=${PAYMENT_ID}` }) as never);
		expect(data).toMatchObject({ payment: { status: 'paid' } });
		apiFetch.mockClear();
		await load(event({ url: 'http://localhost/billing?payment=../admin' }) as never);
		expect(paths().some((p) => p.includes('..'))).toBe(false);
	});

	it('leaves a block out when the backend cannot answer it', async () => {
		apiFetch.mockImplementation(async () => textResponse('nope', 500));
		const data = await load(event({ url: 'http://localhost/billing' }) as never);
		expect(data).toMatchObject({ usage: null, plans: null, subscription: null, entitlements: null, payments: [], referral: null, payment: null });
	});
});

describe('billing: actions', () => {
	it('turns auto-renew on or off', async () => {
		const result = await actions.autoRenew(event({ form: { auto_renew: 'false' } }) as never);
		expect(result).toEqual({ autoRenew: false });
		expect(apiFetch.mock.calls[0][2]).toBe('/api/billing/auto-renew');
		expect(JSON.parse(apiFetch.mock.calls[0][3].body)).toEqual({ auto_renew: false });
	});

	it('cancels a payment by id and shows the backend’s refusal', async () => {
		expect(await actions.cancel(event({ form: { id: PAYMENT_ID } }) as never)).toEqual({ cancelled: PAYMENT_ID });
		apiFetch.mockImplementation(async () => textResponse('already paid', 409));
		expect(await actions.cancel(event({ form: { id: PAYMENT_ID } }) as never)).toMatchObject({ status: 409, data: { error: 'already paid' } });
		expect(await actions.cancel(event({ form: { id: 'x' } }) as never)).toMatchObject({ status: 400 });
	});
});
