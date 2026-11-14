import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(form?: Record<string, string>, reply: (call: BackendCall) => Response = () => json([])) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}
const sent = (api: { calls: BackendCall[] }) => api.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body]);

describe('integrations', () => {
	it('loads Shipyard’s status, its apps, the synced organizations and the webhook endpoints', async () => {
		const { api, event: loading } = event();
		const data = await load(loading);
		expect(api.calls.map((call) => call.url).sort()).toEqual([
			'http://api.test/api/admin/shipyard',
			'http://api.test/api/admin/shipyard/apps',
			'http://api.test/api/admin/shipyard/orgs',
			'http://api.test/api/admin/shipyard/webhooks',
		]);
		expect(data).toMatchObject({ status: { ok: true }, apps: { ok: true }, orgs: { ok: true }, webhooks: { ok: true } });
	});

	it('registers Nomi’s webhook with Shipyard, and catches up on missed events', async () => {
		const registered = event({}, () => json({ configured: true }));
		expect(await actions.registerWebhook(registered.event)).toMatchObject({ action: 'registerWebhook', ok: true });
		const caught = event({}, () => json({ orgs: 2, new_events: 5, errors: [] }));
		expect(await actions.catchUp(caught.event)).toMatchObject({ action: 'catchUp', ok: true, data: { new_events: 5 } });
		expect([...sent(registered.api), ...sent(caught.api)]).toEqual([
			['POST', '/api/admin/shipyard/webhook', undefined],
			['POST', '/api/admin/shipyard/events/catch-up', undefined],
		]);
	});

	it('links an app to a project, or to a new one', async () => {
		const linked = event({ org_id: 'o1', app: 'kasir', project_id: 'p1' }, () => json({}));
		await actions.link(linked.event);
		const fresh = event({ org_id: 'o1', app: 'kasir', project_id: '' }, () => json({}));
		await actions.link(fresh.event);
		expect([...sent(linked.api), ...sent(fresh.api)]).toEqual([
			['POST', '/api/admin/shipyard/apps/link', { org_id: 'o1', app: 'kasir', project_id: 'p1' }],
			['POST', '/api/admin/shipyard/apps/link', { org_id: 'o1', app: 'kasir', project_id: null }],
		]);
	});

	it('syncs and removes a synced organization', async () => {
		const synced = event({ id: 'o1' }, () => json({}));
		await actions.syncOrg(synced.event);
		const removed = event({ id: 'o1' }, () => new Response(null, { status: 204 }));
		await actions.removeOrg(removed.event);
		expect([...sent(synced.api), ...sent(removed.api)]).toEqual([
			['POST', '/api/admin/shipyard/orgs/o1/sync', undefined],
			['DELETE', '/api/admin/shipyard/orgs/o1', undefined],
		]);
	});

	it('adds an endpoint with its events and hands back its secret once', async () => {
		const added = event({ url: ' https://hooks.kopikita.id/nomi ', events: 'publish.succeeded, deployment.*  publish.succeeded', org: '' }, () => json({ id: 'w1', secret: 'whsec_1', own: false }));
		expect(await actions.createEndpoint(added.event)).toMatchObject({ action: 'createEndpoint', ok: true, data: { secret: 'whsec_1' } });
		expect(sent(added.api)).toEqual([['POST', '/api/admin/shipyard/webhooks', { url: 'https://hooks.kopikita.id/nomi', events: ['deployment.*', 'publish.succeeded'], org: null }]]);
		for (const form of [
			{ url: '', events: '*' },
			{ url: 'https://x.id', events: ' ' },
		]) {
			const refused = event(form);
			expect(await actions.createEndpoint(refused.event)).toMatchObject({ status: 400 });
			expect(refused.api.calls).toEqual([]);
		}
	});

	it('switches, tests, rotates and deletes an endpoint', async () => {
		const runs = [
			['toggleEndpoint', { id: 'w1', enabled: '' }, ['PATCH', '/api/admin/shipyard/webhooks/w1', { enabled: false }]],
			['testEndpoint', { id: 'w1' }, ['POST', '/api/admin/shipyard/webhooks/w1/test', undefined]],
			['rotateEndpoint', { id: 'w1' }, ['POST', '/api/admin/shipyard/webhooks/w1/rotate', undefined]],
			['deleteEndpoint', { id: 'w1' }, ['DELETE', '/api/admin/shipyard/webhooks/w1', undefined]],
		] as const;
		for (const [action, form, call] of runs) {
			const { api, event: running } = event(form, () => json({ secret: 'whsec_2' }));
			expect(await actions[action](running)).toMatchObject({ action, ok: true });
			expect(sent(api)).toEqual([call]);
		}
	});
});
