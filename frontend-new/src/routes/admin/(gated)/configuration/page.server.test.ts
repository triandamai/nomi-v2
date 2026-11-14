import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, type BackendCall } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { load, actions } = await import('./+page.server');

function event(form?: Record<string, string>, reply: (call: BackendCall) => Response = () => json({ sections: [], can_manage: true })) {
	const api = fakeBackend(reply);
	return { api, event: fakeEvent({ form, fetch: api.fetch, locals: { accessToken: 't' } }) as never };
}

describe('configuration', () => {
	it('reads every section and where each value comes from', async () => {
		const { api, event: loading } = event();
		expect(await load(loading)).toEqual({ configuration: { ok: true, data: { sections: [], can_manage: true } } });
		expect(api.calls[0].url).toBe('http://api.test/api/admin/configuration');
	});

	it('saves one section’s values, an empty one going back to the environment', async () => {
		const { api, event: saving } = event({ section: 'smtp', value_host: ' mail.kopikita.id ', value_port: '587', value_password: '', value_security: 'starttls' }, () => json({ name: 'smtp', fields: [] }));
		expect(await actions.save(saving)).toMatchObject({ action: 'save', ok: true, data: { section: 'smtp' } });
		expect(api.calls[0]).toMatchObject({ method: 'PUT', url: 'http://api.test/api/admin/configuration/smtp', body: { values: { host: 'mail.kopikita.id', port: '587', password: '', security: 'starttls' } } });
	});

	it('refuses a section it doesn’t know', async () => {
		const { api, event: saving } = event({ section: '../users', value_x: '1' });
		expect(await actions.save(saving)).toMatchObject({ status: 400 });
		expect(api.calls).toEqual([]);
	});

	it('tests storage with what’s typed, and Xendit with what’s saved', async () => {
		const storage = event({ section: 'storage', value_bucket: 'nomi-files', value_secret_access_key: '' }, () => json({ success: false, message: 'bucket not found' }));
		expect(await actions.test(storage.event)).toMatchObject({ action: 'test', ok: true, data: { section: 'storage', success: false, message: 'bucket not found' } });
		expect(storage.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/configuration/storage/test', body: { values: { bucket: 'nomi-files', secret_access_key: '' } } });

		const xendit = event({ section: 'xendit' }, () => json({ success: true, message: 'ok' }));
		expect(await actions.test(xendit.event)).toMatchObject({ data: { section: 'xendit', success: true } });
		expect(xendit.api.calls[0]).toMatchObject({ method: 'POST', url: 'http://api.test/api/admin/payment/xendit/test' });

		const other = event({ section: 'smtp' });
		expect(await actions.test(other.event)).toMatchObject({ status: 400 });
	});
});
