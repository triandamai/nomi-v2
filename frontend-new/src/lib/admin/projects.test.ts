import { describe, expect, it } from 'vitest';
import type { AdminProject } from './api';
import { issueSummary, phaseLabel, sandboxSummary } from './projects';

const project = (over: Partial<AdminProject>): AdminProject => ({
	id: 'p1',
	name: 'warung-kopi-menu',
	owner_email: 'budi@contoh.id',
	workspace: 'Warung Budi',
	runtime: 'sandbox',
	created_at: '2026-11-01T00:00:00Z',
	sandbox_phase: null,
	sandbox_records: 0,
	open_issues: 0,
	issues: 0,
	...over,
});

describe('admin projects', () => {
	it('says when a project never had a sandbox, and how many it has had', () => {
		expect(sandboxSummary(project({}))).toBe('No sandbox');
		expect(sandboxSummary(project({ sandbox_records: 3, sandbox_phase: 'running' }))).toBe('1 active · 3 records');
		expect(sandboxSummary(project({ sandbox_records: 1, sandbox_phase: 'stopped' }))).toBe('0 active · 1 record');
	});

	it('counts open issues', () => {
		expect(issueSummary(project({}))).toBe('No issues');
		expect(issueSummary(project({ issues: 4 }))).toBe('No open issues');
		expect(issueSummary(project({ issues: 4, open_issues: 1 }))).toBe('1 open issue');
		expect(issueSummary(project({ issues: 4, open_issues: 3 }))).toBe('3 open issues');
	});

	it('names a phase, an unknown one as idle', () => {
		expect(phaseLabel('waking')).toBe('Waking up');
		expect(phaseLabel('nope' as never)).toBe('Idle');
	});
});
