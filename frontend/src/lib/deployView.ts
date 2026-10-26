// What the Deployments sheet works out from a project's publishes and a deployment's steps:
// which environments there are, what may be rolled back to, steps grouped by Shipyard's phases,
// and the part of a deployment's log one step wrote.

import type { DeploymentDetail, DeploymentStep, LogLine, ProjectPublish } from '$lib/types';

export type { DeploymentStep, LogLine };

/** Shipyard's phases of a deployment, in the order they run. */
export const PHASE_ORDER = ['validate', 'build', 'configure', 'deploy', 'finalize'] as const;

const DEFAULT_ENVIRONMENT = 'production';
/** Under this many seconds a duration reads in seconds; above, in minutes. */
const SECONDS_SHOWN_UP_TO = 90;
const SECONDS_PER_MINUTE = 60;
const MS_PER_SECOND = 1000;

/** The environments publishes went to: production first, then the rest alphabetically. */
export function environmentsOf(publishes: ProjectPublish[]): string[] {
	const others = [...new Set(publishes.map((p) => p.environment))].filter((e) => e !== DEFAULT_ENVIRONMENT).sort();
	return publishes.some((p) => p.environment === DEFAULT_ENVIRONMENT) ? [DEFAULT_ENVIRONMENT, ...others] : others;
}

/** Whether an admin may go back to this publish: an earlier one that worked and was kept. */
export function canRollBack(publish: ProjectPublish, role: 'admin' | 'member'): boolean {
	return role === 'admin' && publish.status === 'succeeded' && publish.can_roll_back && !publish.is_live;
}

/** Steps by phase, in Shipyard's order; phases Nomi doesn't know come after; empty ones are left out. */
export function groupSteps(steps: DeploymentStep[]): { phase: string; steps: DeploymentStep[] }[] {
	const known: string[] = [...PHASE_ORDER];
	const unknown = [...new Set(steps.map((s) => s.phase).filter((p) => !known.includes(p)))];
	return [...known, ...unknown]
		.map((phase) => ({ phase, steps: steps.filter((s) => s.phase === phase) }))
		.filter((group) => group.steps.length > 0);
}

/** The step to show first: the one that failed, else the one running, else the last that worked. */
export function focusStep(steps: DeploymentStep[]): DeploymentStep | null {
	return (
		steps.find((s) => s.status === 'failed') ??
		steps.find((s) => s.status === 'running') ??
		steps.filter((s) => s.status === 'succeeded').at(-1) ??
		null
	);
}

/** The lines written while `step` ran (Shipyard's log lines don't say their step); null when it never started. */
export function stepLog(logs: LogLine[], step: DeploymentStep): LogLine[] | null {
	if (!step.started_at) return null;
	const from = Date.parse(step.started_at);
	const to = step.finished_at ? Date.parse(step.finished_at) : Infinity;
	return logs.filter((line) => {
		const at = Date.parse(line.at);
		return at >= from && at <= to;
	});
}

/** "42s" or "3 min"; null while it's still going. */
export function durationLabel(startIso: string, endIso: string | null): string | null {
	if (!endIso) return null;
	const seconds = Math.max(0, Math.round((Date.parse(endIso) - Date.parse(startIso)) / MS_PER_SECOND));
	return seconds < SECONDS_SHOWN_UP_TO ? `${seconds}s` : `${Math.round(seconds / SECONDS_PER_MINUTE)} min`;
}

type Raw = Record<string, unknown>;

function logLine(line: unknown): LogLine {
	if (typeof line === 'string') return { level: 'info', message: line, at: '' };
	const l = (line ?? {}) as Raw;
	return { level: String(l.level ?? 'info'), message: String(l.message ?? ''), at: String(l.at ?? '') };
}

function step(value: unknown): DeploymentStep {
	const s = (value ?? {}) as Raw;
	return {
		name: String(s.name ?? ''),
		phase: String(s.phase ?? ''),
		status: String(s.status ?? 'pending'),
		started_at: (s.started_at as string | null | undefined) ?? null,
		finished_at: (s.finished_at as string | null | undefined) ?? null,
		error: (s.error as string | null | undefined) ?? null,
	};
}

/** Shipyard's deployment body (`{ data: … }` or bare), in the shape the sheet uses. */
export function normalizeDeployment(body: unknown, fallbackId: string): DeploymentDetail {
	const outer = (body ?? {}) as Raw;
	const raw = ((outer.data ?? outer.deployment ?? outer) ?? {}) as Raw;
	const logs = raw.logs ?? raw.lines;
	return {
		id: String(raw.deployment_id ?? raw.id ?? fallbackId),
		service: String(raw.service ?? raw.service_name ?? 'web'),
		status: String(raw.status ?? 'unknown'),
		image: (raw.image as string | null | undefined) ?? null,
		created_at: String(raw.created_at ?? ''),
		finished_at: (raw.finished_at as string | null | undefined) ?? null,
		steps: Array.isArray(raw.steps) ? raw.steps.map(step) : [],
		logs: Array.isArray(logs) ? logs.map(logLine) : typeof logs === 'string' ? logs.split('\n').filter(Boolean).map(logLine) : [],
	};
}
