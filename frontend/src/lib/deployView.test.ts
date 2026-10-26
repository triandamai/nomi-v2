import { describe, expect, it } from 'vitest';
import { canRollBack, durationLabel, environmentsOf, focusStep, groupSteps, stepLog, type DeploymentStep } from './deployView';
import type { ProjectPublish } from './types';

function publish(over: Partial<ProjectPublish> = {}): ProjectPublish {
	return {
		id: 'p1',
		environment: 'production',
		status: 'succeeded',
		url: null,
		error: null,
		files_version: 1,
		rollback_of: null,
		rollback_of_version: null,
		started_by_agent: null,
		is_live: false,
		can_roll_back: true,
		created_by: 'Trian',
		created_at: '2026-10-26T09:00:00Z',
		finished_at: '2026-10-26T09:02:00Z',
		...over,
	};
}

// The example deployment from Shipyard's integration docs.
const steps: DeploymentStep[] = [
	{ name: 'clone_repo', phase: 'build', status: 'succeeded', started_at: '2026-10-26T09:00:00Z', finished_at: '2026-10-26T09:00:10Z' },
	{ name: 'build_image', phase: 'build', status: 'failed', started_at: '2026-10-26T09:00:10Z', finished_at: '2026-10-26T09:00:40Z', error: 'npm run build exited with code 1' },
	{ name: 'deploy', phase: 'deploy', status: 'skipped', started_at: null, finished_at: null },
];

describe('environmentsOf', () => {
	it('puts production first, then the rest alphabetically, once each', () => {
		const list = [publish({ environment: 'staging' }), publish(), publish({ environment: 'preview' }), publish({ environment: 'staging' })];
		expect(environmentsOf(list)).toEqual(['production', 'preview', 'staging']);
	});
});

describe('canRollBack', () => {
	it('offers earlier successes to admins only', () => {
		expect(canRollBack(publish(), 'admin')).toBe(true);
		expect(canRollBack(publish(), 'member')).toBe(false);
	});
	it('never offers the live one, a failed one, or one whose source is gone', () => {
		expect(canRollBack(publish({ is_live: true }), 'admin')).toBe(false);
		expect(canRollBack(publish({ status: 'failed' }), 'admin')).toBe(false);
		expect(canRollBack(publish({ status: 'running' }), 'admin')).toBe(false);
		expect(canRollBack(publish({ can_roll_back: false }), 'admin')).toBe(false);
	});
});

describe('steps', () => {
	it('groups steps by phase in Shipyard order, dropping empty phases', () => {
		expect(groupSteps(steps).map((g) => [g.phase, g.steps.map((s) => s.name)])).toEqual([
			['build', ['clone_repo', 'build_image']],
			['deploy', ['deploy']],
		]);
	});
	it('puts unknown phases after the known ones', () => {
		const odd = [{ name: 'x', phase: 'warmup', status: 'succeeded', started_at: null, finished_at: null }, steps[0]];
		expect(groupSteps(odd).map((g) => g.phase)).toEqual(['build', 'warmup']);
	});
	it('focuses the failed step, else the running one, else the last that worked', () => {
		expect(focusStep(steps)?.name).toBe('build_image');
		const running = [steps[0], { ...steps[1], status: 'running', finished_at: null }];
		expect(focusStep(running)?.name).toBe('build_image');
		expect(focusStep([steps[0]])?.name).toBe('clone_repo');
		expect(focusStep([])).toBeNull();
	});
});

describe('stepLog', () => {
	const logs = [
		{ level: 'info', message: 'cloning', at: '2026-10-26T09:00:05Z' },
		{ level: 'info', message: 'vite build', at: '2026-10-26T09:00:20Z' },
		{ level: 'error', message: 'Cannot find module', at: '2026-10-26T09:00:39Z' },
		{ level: 'info', message: 'later', at: '2026-10-26T09:01:00Z' },
	];
	it("keeps the lines written while the step ran", () => {
		expect(stepLog(logs, steps[1])?.map((l) => l.message)).toEqual(['vite build', 'Cannot find module']);
	});
	it('runs open-ended while the step is still going', () => {
		expect(stepLog(logs, { ...steps[1], finished_at: null })?.map((l) => l.message)).toEqual(['vite build', 'Cannot find module', 'later']);
	});
	it('has nothing to say for a step that never started', () => {
		expect(stepLog(logs, steps[2])).toBeNull();
	});
});

describe('durationLabel', () => {
	it('says seconds under a minute and a half, minutes after, nothing while running', () => {
		expect(durationLabel('2026-10-26T09:00:00Z', '2026-10-26T09:00:42Z')).toBe('42s');
		expect(durationLabel('2026-10-26T09:00:00Z', '2026-10-26T09:03:00Z')).toBe('3 min');
		expect(durationLabel('2026-10-26T09:00:00Z', null)).toBeNull();
	});
});

describe('normalizeDeployment', () => {
	it("reads Shipyard's deployment body, steps with their phase and error", async () => {
		const { normalizeDeployment } = await import('./deployView');
		const body = { data: { deployment_id: 'd1', service: 'web', status: 'failed', steps: [{ name: 'build_image', phase: 'build', status: 'failed', error: 'boom' }], logs: [{ level: 'error', message: 'x', at: '2026-10-26T09:00:39Z' }, 'plain line'] } };
		const deployment = normalizeDeployment(body, 'fallback');
		expect(deployment.id).toBe('d1');
		expect(deployment.steps[0]).toEqual({ name: 'build_image', phase: 'build', status: 'failed', started_at: null, finished_at: null, error: 'boom' });
		expect(deployment.logs.map((l) => l.message)).toEqual(['x', 'plain line']);
	});
});
