import { m } from '$lib/paraglide/messages';
import type { PlanBuild, PlanState } from '$lib/types';

/** The plan bubble's status line: how it reads, and how loud. */
export interface PlanStatusLine {
	tone: 'neutral' | 'muted' | 'working' | 'done' | 'attention' | 'error';
	label: string;
	detail: string | null;
}

/**
 * What to say about a plan version, from its state and the build that followed it. It only says
 * what the server recorded: checks passed only when the build's report shows they did. `null`
 * for a server that doesn't report states (nothing to say rather than a guess).
 */
export function planStatusLine(state: PlanState | undefined, build: PlanBuild | null | undefined, latestVersion: number): PlanStatusLine | null {
	switch (state) {
		case 'proposed':
			return { tone: 'neutral', label: m.plan_state_proposed(), detail: null };
		case 'updated':
			return { tone: 'muted', label: m.plan_state_updated(), detail: m.plan_state_updated_detail({ version: String(latestVersion) }) };
		case 'active':
			return { tone: 'working', label: m.plan_state_active(), detail: m.plan_state_active_detail() };
		case 'completed':
			if (!build) return { tone: 'done', label: m.plan_state_done(), detail: m.plan_state_done_detail() };
			if (build.report?.status === 'verified') return { tone: 'done', label: m.plan_state_built(), detail: m.plan_state_verified_detail() };
			if (build.report) return { tone: 'attention', label: m.plan_state_built(), detail: m.plan_state_unverified_detail() };
			return { tone: 'neutral', label: m.plan_state_built(), detail: null };
		case 'blocked': {
			if (build?.status === 'cancelled') return { tone: 'muted', label: m.plan_state_stopped(), detail: m.plan_state_stopped_detail() };
			const failed = build?.report?.checks.find((check) => check.outcome === 'failed');
			if (build?.status === 'completed' && failed) {
				return { tone: 'error', label: m.plan_state_failing(), detail: m.plan_state_failing_detail({ command: failed.command }) };
			}
			return { tone: 'error', label: m.plan_state_blocked(), detail: m.plan_state_blocked_detail() };
		}
		default:
			return null;
	}
}
