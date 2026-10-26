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
};

describe('relay settings', () => {
	it('leaves out database URLs that were not typed, so the stored ones stay', () => {
		const body = settingsBody(form);
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
