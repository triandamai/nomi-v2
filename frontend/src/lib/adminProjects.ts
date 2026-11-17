import { m } from '$lib/paraglide/messages';

// Admin → Projects and Sandboxes (backend routes::admin_projects): the rows, and how a sandbox's
// phase and a project's counts read.

export type SandboxPhase = 'running' | 'waking' | 'stopping' | 'stopped' | 'idle' | 'error';
export type AdminProject = {
	id: string;
	name: string;
	owner_email: string | null;
	workspace: string | null;
	runtime: string;
	created_at: string;
	sandbox_phase: SandboxPhase | null;
	sandbox_records: number;
	open_issues: number;
	issues: number;
};
export type AdminSandbox = {
	id: string;
	project_id: string;
	project_name: string;
	provider: string;
	sandbox_id: string;
	state: string;
	phase: SandboxPhase;
	current: boolean;
	created_at: string;
	updated_at: string;
	ended_at: string | null;
};
export type AdminIssue = { number: number; title: string; kind: string; severity: string; status: 'open' | 'closed'; column_name: string | null; created_at: string; updated_at: string; closed_at: string | null };

const PHASES: Record<SandboxPhase, () => string> = {
	running: m.admin_sandbox_phase_running,
	waking: m.admin_sandbox_phase_waking,
	stopping: m.admin_sandbox_phase_stopping,
	stopped: m.admin_sandbox_phase_stopped,
	idle: m.admin_sandbox_phase_idle,
	error: m.admin_sandbox_phase_error,
};

export const phaseLabel = (phase: SandboxPhase) => (PHASES[phase] ?? PHASES.idle)();

/** A sandbox counts as active while it runs or is on its way up. */
const ACTIVE: SandboxPhase[] = ['running', 'waking'];

export function sandboxSummary(project: AdminProject): string {
	if (!project.sandbox_phase && project.sandbox_records === 0) return m.admin_projects_no_sandbox();
	const active = project.sandbox_phase && ACTIVE.includes(project.sandbox_phase) ? 1 : 0;
	return project.sandbox_records === 1 ? m.admin_projects_sandbox_summary_one({ active }) : m.admin_projects_sandbox_summary({ active, count: project.sandbox_records });
}

export function issueSummary(project: AdminProject): string {
	if (project.issues === 0) return m.admin_projects_issues_none();
	if (project.open_issues === 0) return m.admin_projects_issues_none_open();
	return project.open_issues === 1 ? m.admin_projects_issues_open_one() : m.admin_projects_issues_open({ count: project.open_issues });
}
