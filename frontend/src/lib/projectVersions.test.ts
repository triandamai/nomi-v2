import { describe, expect, it } from 'vitest';
import { canRestore, versionAuthor } from './projectVersions';
import type { ProjectVersion } from '$lib/types';

const version = (extra: Partial<ProjectVersion> = {}): ProjectVersion => ({
	version: 3,
	by: { name: null, agent: 'Koda' },
	at: '2026-11-14T00:00:00Z',
	summary: null,
	files: 4,
	restorable: true,
	restored_from: null,
	current: false,
	...extra,
});

describe('projectVersions', () => {
	it('names the crew member, else the person', () => {
		expect(versionAuthor(version())).toBe('Koda');
		expect(versionAuthor(version({ by: { name: 'ana', agent: null } }))).toBe('ana');
		expect(versionAuthor(version({ by: { name: null, agent: null } }))).toBeNull();
	});

	it('offers restore for kept versions other than the current one', () => {
		expect(canRestore(version())).toBe(true);
		expect(canRestore(version({ current: true }))).toBe(false);
		expect(canRestore(version({ restorable: false }))).toBe(false);
	});
});
