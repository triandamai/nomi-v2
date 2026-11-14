import { describe, expect, it, vi } from 'vitest';
import { fakeBackend, fakeEvent, json, thrown } from '$lib/server/testing/fakeEvent';

vi.mock('$env/dynamic/private', () => ({ env: { API_URL: 'http://api.test' } }));

const { loadAdminShell } = await import('./adminShell');

const whoami = (permissions: string[]) => ({ sub: 'u1', active_org_id: 'o1', permissions, exp: 0, iat: 0 });

describe('loadAdminShell', () => {
	it('reads the person’s admin access from their token, with who they are and their theme', async () => {
		const api = fakeBackend((call) => {
			if (call.url.endsWith('/api/whoami')) return json(whoami(['nomi:admin:plans:[view,write]', 'nomi:o1:member:[view]']));
			if (call.url.endsWith('/api/profile')) return json({ display_name: 'Dinda Rahma', username: null, avatar_url: null, email: 'dinda@nomi.test' });
			if (call.url.endsWith('/api/preferences')) return json({ theme: 'light', language: 'en', timezone: 'Asia/Jakarta', has_stored_timezone: true });
			return json({}, 404);
		});
		const event = fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } });
		expect(await loadAdminShell(event)).toMatchObject({
			access: { plans: 'write' },
			person: { name: 'Dinda Rahma', avatar: null },
			theme: 'light',
			timeZone: 'Asia/Jakarta',
		});
		expect(event.jar.get('nm_theme')).toBe('light');
	});

	it('has no access when whoami can’t be read, and a name even when the profile can’t', async () => {
		const api = fakeBackend(() => json({}, 500));
		expect(await loadAdminShell(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } }))).toMatchObject({ access: {}, person: { name: 'You' }, timeZone: null });
	});

	it('lets a sign-in that can’t be renewed go to sign in', async () => {
		const api = fakeBackend(() => json({}, 401));
		expect(await thrown(() => loadAdminShell(fakeEvent({ fetch: api.fetch, locals: { accessToken: 't' } })))).toMatchObject({ status: 303, location: '/sign-in' });
	});
});
