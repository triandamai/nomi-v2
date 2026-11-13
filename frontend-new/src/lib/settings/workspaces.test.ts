import { describe, expect, it } from 'vitest';
import { workspaceCards } from './workspaces';

const list = {
	orgs: [
		{ id: 'a', name: 'Trian’s space', role: 'admin' as const, members: 3, projects: 4 },
		{ id: 'b', name: 'mario', role: 'owner' as const, members: 1, projects: 1 },
		{ id: 'c', name: 'Kopi Kita', role: 'member' as const, members: 5, projects: 0 },
	],
	project_org_id: 'a',
};

describe('the workspace cards', () => {
	it('gives each its tile, role, people and projects, and marks where projects come from', () => {
		expect(workspaceCards(list)).toEqual([
			{ id: 'a', name: 'Trian’s space', initial: 'T', tone: 'plain', meta: 'Admin · 3 members', projects: 4, members: 3, current: true, href: '/settings/workspaces/a' },
			{ id: 'b', name: 'mario', initial: 'M', tone: 'teal', meta: 'Owner · only you', projects: 1, members: 1, current: false, href: '/settings/workspaces/b' },
			{ id: 'c', name: 'Kopi Kita', initial: 'K', tone: 'amber', meta: 'Member · 5 members', projects: 0, members: 5, current: false, href: '/settings/workspaces/c' },
		]);
	});
});
