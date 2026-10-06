import { m } from '$lib/paraglide/messages';

// Human-readable labels for the raw agent_type/tool_name/phase/status strings that flow through
// the realtime and history APIs. Centralized here so the in-chat status line, the in-chat agent
// activity panel, and the admin command center's table/feed/drill-down all use the same wording
// instead of three independently-drifting copies of the same mapping.

/** Gerund phrases for known tool calls, fit to read naturally after "is" — e.g. "Money is
 * checking your transactions". Covers every tool defined across the engine and every agent
 * (backend/crates/nomi-agent-core/src/engine.rs + nomi-agent-{money,coding,personality,supervisor}). */
const TOOL_LABELS: Record<string, () => string> = {
	// Engine-level tools, available to any agent that opts in
	complete_task: m.tool_complete_task,
	delegate_to_agent: m.tool_delegate_to_agent,
	update_todos: m.tool_update_todos,
	write_plan: m.tool_write_plan,
	// Money agent
	list_transactions: m.tool_list_transactions,
	summarize_budget: m.tool_summarize_budget,
	// Coding agent
	create_project: m.tool_create_project,
	read_file: m.tool_read_file,
	write_file: m.tool_write_file,
	delete_file: m.tool_delete_file,
	list_files: m.tool_list_files,
	// Personality agent
	set_personality: m.tool_set_personality,
	rollback_personality: m.tool_rollback_personality,
	list_personality_versions: m.tool_list_personality_versions,
	// Supervisor agent
	list_recent_agent_activity: m.tool_list_recent_agent_activity,
};

function humanize(raw: string): string {
	return raw.replace(/_/g, ' ');
}

/** A gerund phrase describing what a tool call is doing, e.g. "writing the plan". Falls back to
 * "using {humanized name}" for anything not in the map above — never the raw snake_case name. */
export function toolActivityLabel(toolName: string): string {
	return TOOL_LABELS[toolName]?.() ?? m.tool_using({ tool: humanize(toolName) });
}

/** Best-effort display name for an agent_type with no resolved agent_display_name available.
 * The backend resolves this properly (dynamic agent's configured name, "Nomi" for chitchat,
 * Title-Case for everything else) wherever it can — this exists only as a defensive fallback for
 * call sites that haven't received that field yet (e.g. the admin table's very first paint,
 * before the initial snapshot fetch resolves). */
export function agentTypeFallbackLabel(agentType: string): string {
	if (agentType === 'chitchat') return 'Nomi';
	if (!agentType) return m.agent_an_agent();
	return agentType.charAt(0).toUpperCase() + agentType.slice(1);
}

const PHASE_LABELS: Record<string, () => string> = {
	thinking: m.phase_thinking,
	writing_reply: m.phase_finalizing,
	waiting: m.phase_idle,
};

/** A short label for an agent's current phase, e.g. "finalizing" or "checking your transactions"
 * (for calling_tool, via toolActivityLabel). Used standalone after an agent name: "{name} is
 * {phaseLabel(...)}". */
export function phaseLabel(phase: string, detail: string | null): string {
	if (phase === 'calling_tool') return detail ? toolActivityLabel(detail) : m.tool_using_a_tool();
	return PHASE_LABELS[phase]?.() ?? humanize(phase);
}

const DELEGATION_STATUS_LABELS: Record<string, () => string> = {
	pending: m.status_queued,
	processing: m.status_working,
	claimed: m.status_working,
	completed: m.common_done,
	failed: m.status_failed,
	cancelled: m.status_stopped,
};

/** Friendly label for an agent_delegations.status value. */
export function delegationStatusLabel(status: string): string {
	return DELEGATION_STATUS_LABELS[status]?.() ?? humanize(status);
}

/** The first 8 hex characters of a UUID — enough to visually distinguish concurrent sessions in
 * a feed sentence or table cell without the visual noise of the full id. Callers should also put
 * the full id in a `title` attribute so it's still available on hover/copy. */
export function shortSessionId(sessionId: string): string {
	return sessionId.slice(0, 8);
}

/** One-line sentence for an admin command-center feed entry, e.g. "Planning is finalizing on
 * session a1b2c3d4" or "Money finished session a1b2c3d4". `agentLabel` should already be a
 * resolved display name (or agentTypeFallbackLabel's output) — this function only handles the
 * event-shape-to-sentence mapping, not agent identity resolution.
 *
 * `ToolCalled` covers both a real persisted history row and a live calling_tool phase
 * transition — they're the same semantic event, just observed at different times. `AgentFinalizing`
 * has no persisted counterpart (agent_events never stores phase transitions) — it exists purely
 * as a synthetic marker the live feed uses for a writing_reply transition; the drill-down history
 * (backed only by real agent_events rows) will simply never produce one. */
export function eventFeedLine(
	eventType: string,
	agentLabel: string,
	sessionId: string | null,
	toolName: string | null,
	isError: boolean | null,
): string {
	const where = sessionId ? m.feed_where({ session: shortSessionId(sessionId) }) : '';
	switch (eventType) {
		case 'AgentSpawned':
			return m.feed_picked_up({ agent: agentLabel, where });
		case 'AgentCompleted':
			return m.feed_finished({ agent: agentLabel, where });
		case 'AgentCancelled':
			return m.feed_cancelled({ agent: agentLabel, where });
		case 'AgentExpired':
			return m.feed_expired({ agent: agentLabel, where });
		case 'ToolCalled': {
			const line = m.feed_is_on({ agent: agentLabel, activity: toolName ? toolActivityLabel(toolName) : m.tool_using_a_tool(), where });
			return isError ? m.feed_failed_suffix({ text: line }) : line;
		}
		case 'AgentFinalizing':
			return m.feed_is_on({ agent: agentLabel, activity: m.phase_finalizing(), where });
		case 'AgentReplied':
			return m.feed_replied({ agent: agentLabel, where });
		default:
			return `${agentLabel}: ${eventType}${where}`;
	}
}
