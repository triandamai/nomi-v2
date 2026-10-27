import { describe, expect, it } from 'vitest';
import { originsFromText, readRefusal, settingsBody, type RelaySettingsForm } from './relaySettings';

const form: RelaySettingsForm = {
	public_domain: ' api.relay.example ',
	shared_dev_url: '',
	shared_production_url: 'postgres://relay:pw@db/prod',
	postgrest_dev_url: 'http://pgrst-dev',
	postgrest_production_url: 'http://pgrst-prod',
	postgrest_role: 'authenticator',
	preview_origins: '.webcontainer-api.io\n, .preview.nomi.app',
	smtp_host: '',
	smtp_port: '587',
	smtp_security: 'starttls',
	smtp_user: '',
	smtp_password: '',
	smtp_from: '',
	storage_endpoint: '',
	storage_region: 'auto',
	storage_bucket: '',
	storage_access_key_id: '',
	storage_secret: '',
	storage_path_style: false,
	functions_url: '',
	functions_secret: '',
};

const NOTHING_STORED = { sender: false, storage: false, functions: false };

describe('relay settings', () => {
	it('leaves out database URLs that were not typed, so the stored ones stay', () => {
		const body = settingsBody(form, NOTHING_STORED);
		expect(body).toEqual({
			public_domain: 'api.relay.example',
			shared_production_url: 'postgres://relay:pw@db/prod',
			postgrest_dev_url: 'http://pgrst-dev',
			postgrest_production_url: 'http://pgrst-prod',
			postgrest_role: 'authenticator',
			preview_origins: ['.webcontainer-api.io', '.preview.nomi.app'],
		});
		expect('shared_dev_url' in body).toBe(false);
	});

	it('sends the shared sender when one is typed, keeps a stored password, and clears it when the host is emptied', () => {
		const typed = settingsBody({ ...form, smtp_host: ' smtp.nomi.test ', smtp_user: 'codes', smtp_from: 'Nomi <codes@nomi.test>' }, NOTHING_STORED);
		expect(typed.shared_smtp).toEqual({ host: 'smtp.nomi.test', port: 587, security: 'starttls', user: 'codes', password: '', from: 'Nomi <codes@nomi.test>' });
		expect('shared_smtp' in settingsBody(form, NOTHING_STORED)).toBe(false);
		expect(settingsBody(form, { ...NOTHING_STORED, sender: true }).shared_smtp).toEqual({ host: '', port: 587, security: 'starttls', user: '', password: '', from: '' });
	});

	it('sends file storage when a bucket is typed, keeps a stored secret, and removes it when the bucket is emptied', () => {
		const typed = settingsBody({ ...form, storage_endpoint: ' https://s3.nomi.test/ ', storage_bucket: ' relay-files ', storage_access_key_id: ' AKIA ', storage_path_style: true }, NOTHING_STORED);
		expect(typed.storage).toEqual({ endpoint: 'https://s3.nomi.test', region: 'auto', bucket: 'relay-files', access_key_id: 'AKIA', secret_access_key: '', path_style: true });
		expect('storage' in settingsBody(form, NOTHING_STORED)).toBe(false);
		expect(settingsBody(form, { ...NOTHING_STORED, storage: true }).storage).toEqual({ endpoint: '', region: 'auto', bucket: '', access_key_id: '', secret_access_key: '', path_style: false });
	});

	it('sends the functions runtime when its address is typed, keeps a stored secret, and removes it when emptied', () => {
		const typed = settingsBody({ ...form, functions_url: ' http://relay-functions:8000/ ', functions_secret: 's3cret' }, NOTHING_STORED);
		expect(typed.functions).toEqual({ url: 'http://relay-functions:8000', secret: 's3cret' });
		expect('functions' in settingsBody(form, NOTHING_STORED)).toBe(false);
		expect(settingsBody(form, { ...NOTHING_STORED, functions: true }).functions).toEqual({ url: '', secret: '' });
	});

	it('splits origins on commas and lines', () => {
		expect(originsFromText(' a.test,\n\nb.test , ')).toEqual(['a.test', 'b.test']);
	});

	it("reads Relay's refusal with each check, and anything else as plain text", () => {
		const refusal = readRefusal(new Error(JSON.stringify({ message: "These didn't work: dev database", checks: [{ name: 'dev database', ok: false, message: 'password authentication failed' }] })));
		expect(refusal.message).toContain('dev database');
		expect(refusal.checks[0].message).toBe('password authentication failed');
		expect(readRefusal(new Error('Relay refused the control key'))).toEqual({ message: 'Relay refused the control key', checks: [] });
	});
});
