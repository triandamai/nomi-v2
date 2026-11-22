import { m } from '$lib/paraglide/messages';
import type { AdminProject, SandboxPhase } from './api';

// Admin → Projects and Sandboxes (admin_projects.rs): how a sandbox's phase and a project's
// counts read.

const PHASES: Record<SandboxPhase, () => string> = {
	running: m.admin_sandbox_phase_running,
	waking: m.admin_sandbox_phase_waking,
	stopping: m.admin_sandbox_phase_stopping,
	stopped: m.admin_sandbox_phase_stopped,
	idle: m.admin_sandbox_phase_idle,
	error: m.admin_sandbox_phase_error,
};

export const phaseLabel = (phase: SandboxPhase): string => (PHASES[phase] ?? PHASES.idle)();

/** A sandbox counts as active while it runs or is on its way up. */
const ACTIVE: SandboxPhase[] = ['running', 'waking'];

export function sandboxSummary(project: AdminProject): string {
	if (project.sandbox_records === 0) return project.sandbox_phase ? phaseLabel(project.sandbox_phase) : m.admin_projects_no_sandbox();
	const active = project.sandbox_phase && ACTIVE.includes(project.sandbox_phase) ? 1 : 0;
	return project.sandbox_records === 1 ? m.admin_projects_sandbox_one({ active: String(active) }) : m.admin_projects_sandbox_many({ active: String(active), count: String(project.sandbox_records) });
}

export function issueSummary(project: AdminProject): string {
	if (project.issues === 0) return m.admin_projects_issues_none();
	if (project.open_issues === 0) return m.admin_projects_issues_none_open();
	return project.open_issues === 1 ? m.admin_projects_issues_open_one() : m.admin_projects_issues_open({ count: String(project.open_issues) });
}
