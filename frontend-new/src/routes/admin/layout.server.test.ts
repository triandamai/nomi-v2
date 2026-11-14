import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const shell = await import('./+layout.server');
const gate = await import('./(gated)/+layout.server');

const whoami = (permissions: string[]) => (call: { url: string }) => (call.url.endsWith('/api/whoami') ? json({ sub: 'u1', permissions }) : json({}, 404));

function shellEvent(url: string, answer: Parameters<typeof fakeBackend>[0], signedIn = true) {
	const api = fakeBackend(answer);
	const event = fakeEvent({ url, fetch: api.fetch, locals: signedIn ? { accessToken: 't' } : {} });
	return { api, event: { ...event, depends: () => {}, untrack: <T>(run: () => T) => run() } };
}

const run = (url: string, access: Record<string, string>) => gate.load({ url: new URL(url), parent: async () => ({ access }) } as never);

describe('the admin console’s gate', () => {
	it('sends someone signed out to sign in, and back to the page they wanted', async () => {
		const { api, event } = shellEvent('http://localhost/admin/plans?x=1', () => json({}), false);
		expect(await thrown(() => shell.load(event as never))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Fadmin%2Fplans%3Fx%3D1' });
		expect(api.calls).toEqual([]);
	});

	it('sends someone whose sign-in can’t be renewed to sign in, coming back here', async () => {
		const { event } = shellEvent('http://localhost/admin', () => json({}, 401));
		expect(await thrown(() => shell.load(event as never))).toMatchObject({ status: 303, location: '/sign-in?redirect_to=%2Fadmin' });
	});

	it('never asks whether Nomi is down for maintenance: the console stays open', async () => {
		const { api, event } = shellEvent('http://localhost/admin/maintenance', whoami(['nomi:admin:maintenance:[view,write,manage]']));
		expect(await shell.load(event as never)).toMatchObject({ access: { maintenance: 'manage' } });
		expect(api.calls.map((call) => call.url)).not.toContain('http://api.test/api/maintenance');
	});

	it('says no to someone with no admin area at all', async () => {
		expect(await thrown(() => run('http://localhost/admin', {}))).toMatchObject({ status: 403, body: { message: 'no_admin_access' } });
		expect(await thrown(() => run('http://localhost/admin/plans', {}))).toMatchObject({ status: 403 });
	});

	it('opens a page in an area they have, and sends them from one they don’t to the first they do', async () => {
		expect(await run('http://localhost/admin/plans/new', { plans: 'view' })).toEqual({});
		expect(await thrown(() => run('http://localhost/admin', { plans: 'view' }))).toMatchObject({ status: 303, location: '/admin/plans' });
		expect(await thrown(() => run('http://localhost/admin/users/u2', { plans: 'view', support: 'manage' }))).toMatchObject({ status: 303, location: '/admin/plans' });
		expect(await run('http://localhost/admin', { dashboard: 'view' })).toEqual({});
	});
});
