import { describe, expect, it } from 'vitest';
import { accessFromGrants, accessPayload, adminAccess, areaForPath, can, editable, grantable, homeFor, navGroups, type Access } from './access';

describe('adminAccess', () => {
	it('reads each admin area’s highest level from the token’s permissions', () => {
		expect(
			adminAccess([
				'nomi:admin:plans:[view]',
				'nomi:admin:user:[view,write]',
				'nomi:admin:models:[manage]',
				'nomi:2f0c…:member:[view,manage]',
				'nomi:admin:nowhere:[manage]',
				'nomi:admin:tools:[own]',
				'something else',
			]),
		).toEqual({ plans: 'view', user: 'write', models: 'manage' });
	});

	it('keeps the higher level when an area is listed twice', () => {
		expect(adminAccess(['nomi:admin:support:[manage]', 'nomi:admin:support:[view]'])).toEqual({ support: 'manage' });
	});

	it('gives a platform admin every area at manage, as the backend writes it', () => {
		const all = ['dashboard', 'user', 'plans', 'notifications', 'models', 'tools', 'agents', 'support', 'maintenance', 'integrations', 'configuration'].map((area) => `nomi:admin:${area}:[view,write,manage]`);
		expect(Object.values(adminAccess(all))).toEqual(Array(11).fill('manage'));
	});
});

describe('can', () => {
	const access: Access = { plans: 'write', user: 'view' };

	it('includes the levels below the one held', () => {
		expect(can(access, 'plans')).toBe(true);
		expect(can(access, 'plans', 'view')).toBe(true);
		expect(can(access, 'plans', 'write')).toBe(true);
		expect(can(access, 'plans', 'manage')).toBe(false);
		expect(can(access, 'user', 'write')).toBe(false);
		expect(can(access, 'models')).toBe(false);
	});
});

describe('pages and areas', () => {
	it('knows the area of every admin page, the overview being the dashboard’s', () => {
		expect(areaForPath('/admin')).toBe('dashboard');
		expect(areaForPath('/admin/')).toBe('dashboard');
		expect(areaForPath('/admin/agents')).toBe('dashboard');
		expect(areaForPath('/admin/usage')).toBe('dashboard');
		expect(areaForPath('/admin/users/7c1d')).toBe('user');
		expect(areaForPath('/admin/plans/new')).toBe('plans');
		expect(areaForPath('/admin/models/providers')).toBe('models');
		expect(areaForPath('/admin/dynamic-agents/new')).toBe('agents');
		expect(areaForPath('/admin/configuration')).toBe('configuration');
		expect(areaForPath('/admin/integrations/apps/1/web')).toBe('integrations');
		expect(areaForPath('/admin/usagelike')).toBeNull();
		expect(areaForPath('/admin/nowhere')).toBeNull();
	});

	it('sends someone to the first page they can see', () => {
		expect(homeFor({ dashboard: 'view', plans: 'view' })).toBe('/admin');
		expect(homeFor({ plans: 'view', support: 'manage' })).toBe('/admin/plans');
		expect(homeFor({ configuration: 'view' })).toBe('/admin/configuration');
		expect(homeFor({})).toBeNull();
	});

	it('lists only the pages someone can see, in groups, leaving out empty groups', () => {
		const groups = navGroups({ plans: 'view', tools: 'write' });
		expect(groups.map((group) => group.key)).toEqual(['people', 'crew']);
		expect(groups.flatMap((group) => group.pages.map((page) => page.href))).toEqual(['/admin/plans', '/admin/tools']);
		expect(navGroups({}).length).toBe(0);
		expect(
			navGroups({ dashboard: 'view', user: 'view', plans: 'view', notifications: 'view', models: 'view', tools: 'view', agents: 'view', support: 'view', maintenance: 'view', integrations: 'view', configuration: 'view' }).flatMap(
				(group) => group.pages.map((page) => page.area),
			),
		).toEqual(['dashboard', 'dashboard', 'dashboard', 'user', 'plans', 'notifications', 'models', 'models', 'models', 'tools', 'agents', 'support', 'maintenance', 'integrations', 'configuration']);
	});
});

describe('granting access', () => {
	it('offers only the levels the granter holds on that area', () => {
		expect(grantable({ plans: 'write' }, 'plans')).toEqual(['view', 'write']);
		expect(grantable({ plans: 'manage' }, 'plans')).toEqual(['view', 'write', 'manage']);
		expect(grantable({ plans: 'manage' }, 'tools')).toEqual([]);
	});

	it('lets an area be changed only when the granter holds both its current level and the new one', () => {
		expect(editable({ plans: 'write' }, 'plans', 'view')).toBe(true);
		expect(editable({ plans: 'write' }, 'plans', null)).toBe(true);
		expect(editable({ plans: 'write' }, 'plans', 'manage')).toBe(false);
		expect(editable({}, 'plans', null)).toBe(false);
	});

	it('reads someone’s admin role from their grants, ignoring workspace grants', () => {
		expect(
			accessFromGrants([
				{ id: '1', scope_type: 'admin', org_id: null, org_name: null, resource: 'plans', actions: ['view', 'write'], created_at: '' },
				{ id: '2', scope_type: 'admin', org_id: null, org_name: null, resource: 'support', actions: ['manage'], created_at: '' },
				{ id: '3', scope_type: 'org', org_id: 'o', org_name: 'Kopi', resource: 'member', actions: ['manage'], created_at: '' },
				{ id: '4', scope_type: 'admin', org_id: null, org_name: null, resource: 'retired', actions: ['view'], created_at: '' },
			]),
		).toEqual({ plans: 'write', support: 'manage' });
	});

	it('sends the whole role in the backend’s order', () => {
		expect(accessPayload({ support: 'manage', plans: 'view', tools: null })).toEqual({
			access: [
				{ resource: 'plans', level: 'view' },
				{ resource: 'support', level: 'manage' },
			],
		});
	});
});
