import { beforeEach, describe, expect, it, vi } from 'vitest';
import { event, json, textResponse } from '../../../../test-stubs/server';

const apiFetch = vi.fn();
vi.mock('$lib/server/api', () => ({ apiFetch: (...args: unknown[]) => apiFetch(...args) }));

const { actions, load } = await import('./+page.server');

const calls = () => apiFetch.mock.calls.map(([, , path, init]) => ({ path, method: init?.method ?? 'GET', body: init?.body ? JSON.parse(init.body) : undefined }));
const status = (result: unknown) => (result as { status?: number }).status;

beforeEach(() => {
	apiFetch.mockReset();
	apiFetch.mockImplementation(async () => json({ id: 'u1' }));
});

describe('admin users: load', () => {
	it('pages and searches, with a page size from the list only', async () => {
		apiFetch.mockImplementation(async (_f: unknown, _c: unknown, path: string) =>
			path.startsWith('/api/admin/users') ? json({ users: [{ id: 'u1' }], total: 41 }) : json([{ id: 'free' }]),
		);
		const data = (await load(event({ url: 'http://localhost/admin/users?query=ana&page=3&size=50' }))) as Record<string, unknown>;
		expect(data).toMatchObject({ users: [{ id: 'u1' }], total: 41, page: 3, pageSize: 50, query: 'ana', plans: [{ id: 'free' }] });
		expect(calls()[0].path).toBe('/api/admin/users?page=3&page_size=50&query=ana');

		const fallback = (await load(event({ url: 'http://localhost/admin/users?page=-2&size=7' }))) as Record<string, unknown>;
		expect(fallback).toMatchObject({ page: 1, pageSize: 20, query: '' });
	});

	it('shows an empty list when the API fails', async () => {
		apiFetch.mockImplementation(async () => json({}, 500));
		expect(await load(event())).toMatchObject({ users: [], total: 0, plans: [] });
	});
});

describe('admin users: actions', () => {
	const cases: [keyof typeof actions, Record<string, string>, string, string][] = [
		['loadSubscription', { userId: 'u1' }, '/api/admin/users/u1/subscription', 'GET'],
		['promote', { userId: 'u1' }, '/api/admin/users/u1/permissions', 'POST'],
		['loadUserDetail', { userId: 'u1' }, '/api/admin/users/u1', 'GET'],
		['updateUser', { userId: 'u1', display_name: 'Ana', username: 'ana' }, '/api/admin/users/u1/profile', 'PUT'],
		['revokePermission', { userId: 'u1', permissionId: 'p1' }, '/api/admin/users/u1/permissions/p1', 'DELETE'],
		['saveSubscription', { userId: 'u1', plan_id: 'pro' }, '/api/admin/users/u1/subscription', 'PUT'],
	];

	for (const [name, form, path, method] of cases) {
		it(`${name}: calls the API, refuses a missing user, and passes failures on`, async () => {
			const ok = await actions[name](event({ form }));
			expect(status(ok)).toBeUndefined();
			expect(calls()[0]).toMatchObject({ path, method });

			expect(status(await actions[name](event({ form: {} })))).toBe(400);

			apiFetch.mockImplementationOnce(async () => textResponse('nope', 403));
			expect(await actions[name](event({ form }))).toMatchObject({ status: 403, data: { error: 'nope' } });
			apiFetch.mockImplementationOnce(async () => textResponse('', 500));
			const fallback = (await actions[name](event({ form }))) as { data: { error: string } };
			expect(fallback.data.error).toBeTruthy();
		});
	}

	it('saveSubscription: an allowance must be a whole number, and an end date goes with it', async () => {
		expect(status(await actions.saveSubscription(event({ form: { userId: 'u1', plan_id: 'pro', quota_override: '1.5' } })))).toBe(400);
		expect(status(await actions.saveSubscription(event({ form: { userId: 'u1', plan_id: 'pro', quota_override: '-1' } })))).toBe(400);
		await actions.saveSubscription(event({ form: { userId: 'u1', plan_id: 'pro', quota_override: '500000', override_until: '2026-12-31', note: ' bonus ' } }));
		const body = calls()[0].body;
		expect(body).toMatchObject({ plan_id: 'pro', quota_override: 500000, note: 'bonus' });
		expect(body.override_until).toMatch(/^2026-12-31T/);
		await actions.saveSubscription(event({ form: { userId: 'u1', plan_id: 'pro', override_until: '2026-12-31' } }));
		expect(calls()[1].body).toMatchObject({ quota_override: null, override_until: null, note: null });
	});
});
