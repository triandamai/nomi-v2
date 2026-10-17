import { describe, expect, it } from 'vitest';
import {
	ALLOWED_PROJECT_API_PATHS,
	canTransferProject,
	filterTransferOrgs,
	formatTransferNoticeParams,
} from './projectTransfer';
import type { TransferOrgSummary } from './types';

describe('projectTransfer helpers', () => {
	const mockOrgs: TransferOrgSummary[] = [
		{
			id: 'org-1',
			name: 'Acme Corp',
			is_personal: false,
			owner_id: 'user-1',
			owner_name: 'Alice Smith',
			my_role: 'admin',
		},
		{
			id: 'org-2',
			name: 'Bob Personal',
			is_personal: true,
			owner_id: 'user-2',
			owner_name: 'Bob Jones',
			my_role: 'owner',
		},
		{
			id: 'org-3',
			name: 'Beta Labs',
			is_personal: false,
			owner_id: null,
			owner_name: null,
			my_role: 'member',
		},
	];

	describe('filterTransferOrgs', () => {
		it('returns all organizations if query is empty or whitespace', () => {
			expect(filterTransferOrgs(mockOrgs, '')).toEqual(mockOrgs);
			expect(filterTransferOrgs(mockOrgs, '   ')).toEqual(mockOrgs);
		});

		it('filters organizations case-insensitively by name', () => {
			const result = filterTransferOrgs(mockOrgs, 'acme');
			expect(result).toHaveLength(1);
			expect(result[0].id).toBe('org-1');

			const bResult = filterTransferOrgs(mockOrgs, 'b');
			expect(bResult.map((o) => o.id)).toEqual(['org-2', 'org-3']);
		});

		it('returns empty array when no organizations match', () => {
			expect(filterTransferOrgs(mockOrgs, 'NonExistent')).toEqual([]);
		});
	});

	describe('canTransferProject', () => {
		it('allows project owner regardless of org role', () => {
			expect(canTransferProject({ isProjectOwner: true, orgRole: 'member' })).toBe(true);
			expect(canTransferProject({ isProjectOwner: true, orgRole: undefined })).toBe(true);
		});

		it('allows org owner or admin even if not direct project owner', () => {
			expect(canTransferProject({ isProjectOwner: false, orgRole: 'owner' })).toBe(true);
			expect(canTransferProject({ isProjectOwner: false, orgRole: 'admin' })).toBe(true);
		});

		it('denies members and guests who are not the project owner', () => {
			expect(canTransferProject({ isProjectOwner: false, orgRole: 'member' })).toBe(false);
			expect(canTransferProject({ isProjectOwner: false, orgRole: 'viewer' })).toBe(false);
			expect(canTransferProject({ isProjectOwner: false, orgRole: null })).toBe(false);
		});
	});

	describe('formatTransferNoticeParams', () => {
		it('formats org and owner names correctly', () => {
			expect(formatTransferNoticeParams(mockOrgs[0])).toEqual({
				org: 'Acme Corp',
				owner: 'Alice Smith',
			});
		});

		it('falls back to dash when owner name is null or whitespace', () => {
			expect(formatTransferNoticeParams(mockOrgs[2])).toEqual({
				org: 'Beta Labs',
				owner: '—',
			});
		});
	});

	describe('proxy route ALLOWED_PROJECT_API_PATHS pattern', () => {
		it('allows transfer and transfer/orgs', () => {
			expect(ALLOWED_PROJECT_API_PATHS.test('transfer')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('transfer/orgs')).toBe(true);
		});

		it('rejects unauthorized paths under transfer', () => {
			expect(ALLOWED_PROJECT_API_PATHS.test('transfer/nested/path')).toBe(false);
			expect(ALLOWED_PROJECT_API_PATHS.test('transfer/delete')).toBe(false);
			expect(ALLOWED_PROJECT_API_PATHS.test('transfer/..')).toBe(false);
		});

		it('still allows all standard project endpoints', () => {
			expect(ALLOWED_PROJECT_API_PATHS.test('')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('runtime')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('deploy')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('issues')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('members')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('database')).toBe(true);
			expect(ALLOWED_PROJECT_API_PATHS.test('showcase')).toBe(true);
		});
	});
});
