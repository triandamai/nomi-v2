import { describe, expect, it } from 'vitest';
import { adminAccess, areaForPath, can, highest, searchAreas } from './permissions';

describe('adminAccess', () => {
	it('reads each admin area at its highest level', () => {
		const access = adminAccess([
			'nomi:admin:models:[view,write]',
			'nomi:admin:integrations:[view,write,manage]',
			'nomi:11111111-1111-1111-1111-111111111111:member:[view,manage]',
		]);
		expect(access).toEqual({ models: 'write', integrations: 'manage' });
	});
});

describe('can', () => {
	const access = { support: 'write' } as const;
	it('includes the levels below', () => {
		expect(can(access, 'support', 'view')).toBe(true);
		expect(can(access, 'support', 'write')).toBe(true);
		expect(can(access, 'support', 'manage')).toBe(false);
	});
	it('is false for an area they have nothing on', () => {
		expect(can(access, 'models')).toBe(false);
	});
});

describe('highest', () => {
	it('ignores actions that are not levels', () => {
		expect(highest(['view', 'other'])).toBe('view');
		expect(highest(['other'])).toBeNull();
	});
});

describe('areaForPath', () => {
	it('maps admin pages to their area', () => {
		expect(areaForPath('/admin')?.resource).toBe('dashboard');
		expect(areaForPath('/admin/settings/embedding')?.resource).toBe('models');
		expect(areaForPath('/admin/integrations')?.resource).toBe('integrations');
		expect(areaForPath('/admin/users')?.resource).toBe('user');
		expect(areaForPath('/admin/api/shipyard')).toBeUndefined();
	});
});

describe('searchAreas', () => {
	it('finds areas by key and puts label prefix matches first', () => {
		expect(searchAreas('integ')[0].resource).toBe('integrations');
		expect(searchAreas('maintenance').map((a) => a.resource)).toContain('maintenance');
		expect(searchAreas('zzz')).toEqual([]);
	});
});
