import { describe, expect, it } from 'vitest';
import { isBroken, needsAttention, needsYou } from './projectHealth';
import type { ProjectSummary } from '$lib/types';

const project = (extra: Partial<ProjectSummary> = {}): ProjectSummary => ({
	id: 'p',
	session_id: 's',
	name: 'Kopi',
	description: null,
	status: 'ready',
	created_at: '2026-11-01T00:00:00Z',
	updated_at: '2026-11-01T00:00:00Z',
	created_by: 'ana',
	open_issues: 0,
	...extra,
});

describe('projectHealth', () => {
	it('counts what waits on the person, none when the backend says nothing', () => {
		expect(needsYou(project())).toBe(0);
		expect(needsYou(project({ waiting_on_you: 2 }))).toBe(2);
	});

	it('is broken when the live app is down or the last publish failed', () => {
		expect(isBroken(project({ live_status: 'up', last_deploy: { status: 'ok', at: '' } }))).toBe(false);
		expect(isBroken(project({ live_status: 'down' }))).toBe(true);
		expect(isBroken(project({ last_deploy: { status: 'failed', at: '' } }))).toBe(true);
	});

	it('needs attention for issues, questions or a broken app', () => {
		expect(needsAttention(project())).toBe(false);
		expect(needsAttention(project({ open_issues: 1 }))).toBe(true);
		expect(needsAttention(project({ waiting_on_you: 1 }))).toBe(true);
		expect(needsAttention(project({ live_status: 'down' }))).toBe(true);
	});
});
