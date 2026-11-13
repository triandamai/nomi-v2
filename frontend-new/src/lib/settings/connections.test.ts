import { describe, expect, it } from 'vitest';
import { appsToAdd, connectedApps, maskAddress } from './connections';

const overview = {
	google: { configured: true, email: 'kotaksurat@gmail.com', services: ['calendar', 'gmail'] },
	hevy: { provider: 'hevy', account_name: 'mario', connected_at: '2026-10-01T00:00:00Z', last_used_at: '2026-10-20T00:00:00Z' },
};
const github = { oauth: true, connection: { login: 'marioprasetyo', kind: 'oauth', created_at: '2026-10-01T00:00:00Z', last_used_at: null } };

describe('the apps on Connections', () => {
	it('lists what’s connected: Google with the services granted, GitHub, Hevy, and who uses each', () => {
		const apps = connectedApps(overview, github);
		expect(apps.map((app) => [app.id, app.account, app.usedBy[0].name])).toEqual([
			['google', 'ko••@gmail.com', 'Tara'],
			['github', '@marioprasetyo', 'Koda'],
			['hevy', 'mario', 'Raga'],
		]);
		expect(apps[0]).toMatchObject({ caption: 'Gmail and Calendar', can: ['Gmail', 'Calendar'] });
	});

	it('offers the rest to add, and Google only where it’s set up', () => {
		expect(appsToAdd({ google: { configured: false, email: null, services: [] }, hevy: null }, { oauth: false, connection: null }).map((app) => [app.id, app.available])).toEqual([
			['google', false],
			['github', true],
			['hevy', true],
		]);
		expect(appsToAdd(overview, github)).toEqual([]);
	});

	it('hides most of the address', () => {
		expect(maskAddress('kotaksurat@gmail.com')).toBe('ko••@gmail.com');
	});
});
