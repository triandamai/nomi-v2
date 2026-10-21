import { describe, expect, it } from 'vitest';
import { planStatusLine } from './planStatus';
import type { BuildReport, PlanBuild } from './types';

function build(status: string, report: BuildReport | null = null, error: string | null = null): PlanBuild {
	return { delegation_id: 'd1', status, report, error, updated_at: '2026-10-21T10:00:00Z' };
}

function report(status: BuildReport['status'], checks: BuildReport['checks'] = []): BuildReport {
	return { status, files_version: 4, changed_files: ['src/app.ts'], deleted_files: [], checks };
}

describe('planStatusLine', () => {
	it('says nothing for a server that doesn’t report plan states', () => {
		expect(planStatusLine(undefined, undefined, 1)).toBeNull();
	});

	it('shows a plan nobody has acted on as proposed', () => {
		expect(planStatusLine('proposed', null, 1)).toEqual({ tone: 'neutral', label: 'Proposed', detail: null });
	});

	it('points an older version at the latest one', () => {
		expect(planStatusLine('updated', null, 3)).toEqual({ tone: 'muted', label: 'Updated', detail: 'Version 3 is the latest' });
	});

	it('shows a running build as in progress', () => {
		expect(planStatusLine('active', build('processing'), 1)).toEqual({ tone: 'working', label: 'In progress', detail: 'Koda is building it' });
	});

	it('only says the checks passed when the report shows they did', () => {
		expect(planStatusLine('completed', build('completed', report('verified')), 1)).toEqual({
			tone: 'done',
			label: 'Built',
			detail: 'Checks passed on the final files',
		});
		expect(planStatusLine('completed', build('completed', report('unverified')), 1)).toEqual({
			tone: 'attention',
			label: 'Built',
			detail: 'Not verified yet: the checks haven’t passed on the final files',
		});
		// Built before reports existed: no claim either way.
		expect(planStatusLine('completed', build('completed'), 1)).toEqual({ tone: 'neutral', label: 'Built', detail: null });
	});

	it('shows a plan the person ticked off as done', () => {
		expect(planStatusLine('completed', null, 1)).toEqual({ tone: 'done', label: 'Done', detail: 'Every step is ticked off' });
	});

	it('names the check that failed, a build that broke, and one that was stopped', () => {
		const failing = report('failing', [
			{ command: 'npm run check', outcome: 'passed', source: 'koda', exit_code: 0, output_tail: null },
			{ command: 'npm run build', outcome: 'failed', source: 'koda', exit_code: 1, output_tail: 'boom' },
		]);
		expect(planStatusLine('blocked', build('completed', failing), 1)).toEqual({
			tone: 'error',
			label: 'Not working yet',
			detail: 'npm run build failed on the final files',
		});
		expect(planStatusLine('blocked', build('failed', null, 'llm call failed: 500'), 1)).toEqual({
			tone: 'error',
			label: 'Blocked',
			detail: 'Koda couldn’t finish the build',
		});
		expect(planStatusLine('blocked', build('cancelled'), 1)).toEqual({ tone: 'muted', label: 'Stopped', detail: 'The build was stopped before it finished' });
	});
});
