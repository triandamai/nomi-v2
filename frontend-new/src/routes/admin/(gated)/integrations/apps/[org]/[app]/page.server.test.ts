import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(form?: Record<string, string>, reply: (call: BackendCall) => Response = () => json({ key: 'kasir', environments: [] })) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ params: { org: 'o1', app: 'kasir' }, form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}
const sent = (api: { calls: BackendCall[] }) => api.calls.map((call) => [call.method, call.url.replace('http://api.test', ''), call.body]);
const base = '/api/admin/shipyard/orgs/o1/apps/kasir';

describe('a Shipyard app', () => {
	it('loads its environments and its secrets', async () => {
		const { api, event: loading } = event(undefined, (call) => (call.url.endsWith('/secrets') ? json([{ name: 'DATABASE_URL' }]) : json({ key: 'kasir', environments: [] })));
		expect(await load(loading)).toMatchObject({ title: 'kasir', app: { ok: true }, secrets: { ok: true, data: [{ name: 'DATABASE_URL' }] } });
		expect(api.calls.map((call) => call.url).sort()).toEqual([`http://api.test${base}`, `http://api.test${base}/secrets`]);
	});

	it('duplicates an environment, with or without its secrets, and deletes one', async () => {
		const copied = event({ from: 'production', to: ' staging ', copy_secrets: 'on' }, () => json({}));
		await actions.duplicateEnv(copied.event);
		const bare = event({ from: 'production', to: 'preview', copy_secrets: '' }, () => json({}));
		await actions.duplicateEnv(bare.event);
		const deleted = event({ env: 'preview' }, () => new Response(null, { status: 204 }));
		await actions.deleteEnv(deleted.event);
		expect([...sent(copied.api), ...sent(bare.api), ...sent(deleted.api)]).toEqual([
			['POST', `${base}/envs/production/duplicate`, { to: 'staging', copy_secrets: true }],
			['POST', `${base}/envs/production/duplicate`, { to: 'preview', copy_secrets: false }],
			['DELETE', `${base}/envs/preview`, undefined],
		]);
		const unnamed = event({ from: 'production', to: '' });
		expect(await actions.duplicateEnv(unnamed.event)).toMatchObject({ status: 400 });
	});

	it('sets a secret (one Shipyard makes up when left empty) for every environment or one, and deletes it', async () => {
		const typed = event({ name: 'MIDTRANS_KEY', value: 'sk-1', environment: 'production' }, () => json({}));
		await actions.setSecret(typed.event);
		const generated = event({ name: 'SESSION_SECRET', value: '', environment: '' }, () => json({}));
		await actions.setSecret(generated.event);
		const deleted = event({ name: 'MIDTRANS_KEY', environment: 'production' }, () => new Response(null, { status: 204 }));
		await actions.deleteSecret(deleted.event);
		expect([...sent(typed.api), ...sent(generated.api), ...sent(deleted.api)]).toEqual([
			['PUT', `${base}/secrets/MIDTRANS_KEY`, { value: 'sk-1', environment: 'production' }],
			['PUT', `${base}/secrets/SESSION_SECRET`, {}],
			['DELETE', `${base}/secrets/MIDTRANS_KEY?environment=production`, undefined],
		]);
		const lower = event({ name: 'midtrans', value: 'x' });
		expect(await actions.setSecret(lower.event)).toMatchObject({ status: 400 });
	});

	it('deletes the app and goes back to integrations', async () => {
		const { api, event: deleting } = event({}, () => new Response(null, { status: 204 }));
		expect(await thrown(() => actions.deleteApp(deleting))).toMatchObject({ status: 303, location: '/admin/integrations' });
		expect(sent(api)).toEqual([['DELETE', base, undefined]]);
	});
});
