import { describe, expect, it } from 'vitest';
import type { OrgDetail } from '$lib/adapters/api';
import { canLeave, inviteRows, memberRows, parseEmails } from './workspace';

const now = new Date('2026-11-13T10:00:00Z');
const detail = (my_role: OrgDetail['my_role'], members: OrgDetail['members'] = []): OrgDetail => ({ id: 'w', name: 'Kopi', my_role, me: 'me', members, invites: [] });
const person = (user_id: string, role: OrgDetail['my_role'], name = user_id) => ({ user_id, name, email: `${name}@kopi.id`, avatar_url: null, role, joined_at: '2026-07-14T00:00:00Z' });

describe('inviting addresses', () => {
	it('splits on commas, spaces and new lines, keeps each once, and sets the odd ones aside', () => {
		expect(parseEmails('Siti@kopi.id, budi.s@gmail.com\nsiti@kopi.id;  nope ,x@y')).toEqual({ valid: ['siti@kopi.id', 'budi.s@gmail.com'], invalid: ['nope', 'x@y'] });
		expect(parseEmails('  ')).toEqual({ valid: [], invalid: [] });
	});
});

describe('the members', () => {
	const members = [person('owner1', 'owner', 'trian'), person('me', 'admin', 'mario'), person('m1', 'member', 'siti')];

	it('lets an admin move members and admins, never an owner or themselves', () => {
		const rows = memberRows(detail('admin', members), now);
		expect(rows.map((row) => [row.name, row.you, row.roleOptions, row.canRemove])).toEqual([
			['trian', false, [], false],
			['mario', true, [], false],
			['siti', false, ['admin', 'member'], true],
		]);
		expect(rows[2].meta).toBe('si••@kopi.id · joined 14 Jul');
	});

	it('lets an owner make owners too', () => {
		expect(memberRows(detail('owner', [person('me', 'owner'), person('m1', 'member')]), now)[1].roleOptions).toEqual(['owner', 'admin', 'member']);
	});

	it('gives a member nothing to change', () => {
		expect(memberRows(detail('member', members), now).every((row) => !row.roleOptions.length && !row.canRemove)).toBe(true);
	});
});

describe('open invites and leaving', () => {
	it('says who was invited, when, and how long it has left', () => {
		const rows = inviteRows([{ code: 'c', email: 'andi@kopikita.id', role: 'member', created_at: '2026-11-11T10:00:00Z', expires_at: '2026-11-18T09:00:00Z' }], now);
		expect(rows).toEqual([{ code: 'c', email: 'andi@kopikita.id', role: 'member', meta: 'invited 2d ago · expires in 5 days' }]);
		expect(inviteRows([{ code: 'c', email: 'a@b.co', role: 'admin', created_at: '2026-11-13T09:59:40Z', expires_at: '2026-11-20T10:00:00Z' }], now)[0].meta).toBe('invited just now · expires in 7 days');
	});

	it('keeps the last owner from leaving', () => {
		expect(canLeave(detail('owner', [person('me', 'owner'), person('m1', 'member')]))).toBe(false);
		expect(canLeave(detail('owner', [person('me', 'owner'), person('o2', 'owner')]))).toBe(true);
		expect(canLeave(detail('member', [person('o', 'owner'), person('me', 'member')]))).toBe(true);
	});
});
