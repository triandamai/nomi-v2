// Human-readable labels for the raw agent_type/tool_name/phase/status strings that flow through
// the realtime and history APIs. Centralized here so the in-chat status line, the in-chat agent
// activity panel, and the admin command center's table/feed/drill-down all use the same wording
// instead of three independently-drifting copies of the same mapping.

/** Gerund phrases for known tool calls, fit to read naturally after "is" — e.g. "Money is
 * checking your transactions". Covers every tool defined across the engine and every agent
 * (backend/crates/nomi-agent-core/src/engine.rs + nomi-agent-{money,coding,personality,supervisor}). */
const TOOL_LABELS: Record<string, string> = {
	// Engine-level tools, available to any agent that opts in
	complete_task: 'wrapping up',
	delegate_to_agent: 'delegating to another agent',
	update_todos: 'updating the to-do list',
	write_plan: 'writing the plan',
	// Money agent
	list_transactions: 'checking your transactions',
	summarize_budget: 'summarizing your budget',
	// Coding agent
	create_project: 'setting up the project',
	read_file: 'reading a file',
	write_file: 'writing a file',
	delete_file: 'deleting a file',
	list_files: 'listing files',
	// Personality agent
	set_personality: 'updating its personality',
	rollback_personality: 'rolling back its personality',
	list_personality_versions: 'checking personality history',
	// Supervisor agent
	list_recent_agent_activity: 'checking recent agent activity',
};

function humanize(raw: string): string {
	return raw.replace(/_/g, ' ');
}

/** A gerund phrase describing what a tool call is doing, e.g. "writing the plan". Falls back to
 * "using {humanized name}" for anything not in the map above — never the raw snake_case name. */
export function toolActivityLabel(toolName: string): string {
	return TOOL_LABELS[toolName] ?? `using ${humanize(toolName)}`;
}

/** Best-effort display name for an agent_type with no resolved agent_display_name available.
 * The backend resolves this properly (dynamic agent's configured name, "Nomi" for chitchat,
 * Title-Case for everything else) wherever it can — this exists only as a defensive fallback for
 * call sites that haven't received that field yet (e.g. the admin table's very first paint,
 * before the initial snapshot fetch resolves). */
export function agentTypeFallbackLabel(agentType: string): string {
	if (agentType === 'chitchat') return 'Nomi';
	if (!agentType) return 'An agent';
	return agentType.charAt(0).toUpperCase() + agentType.slice(1);
}

const PHASE_LABELS: Record<string, string> = {
	thinking: 'thinking',
	writing_reply: 'finalizing',
	waiting: 'idle',
};

/** A short label for an agent's current phase, e.g. "finalizing" or "checking your transactions"
 * (for calling_tool, via toolActivityLabel). Used standalone after an agent name: "{name} is
 * {phaseLabel(...)}". */
export function phaseLabel(phase: string, detail: string | null): string {
	if (phase === 'calling_tool') return detail ? toolActivityLabel(detail) : 'using a tool';
	return PHASE_LABELS[phase] ?? humanize(phase);
}

const DELEGATION_STATUS_LABELS: Record<string, string> = {
	pending: 'Queued',
	processing: 'Working…',
	claimed: 'Working…',
	completed: 'Done',
	failed: 'Failed',
};

/** Friendly label for an agent_delegations.status value. */
export function delegationStatusLabel(status: string): string {
	return DELEGATION_STATUS_LABELS[status] ?? humanize(status);
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
	const where = sessionId ? ` session ${shortSessionId(sessionId)}` : '';
	switch (eventType) {
		case 'AgentSpawned':
			return `${agentLabel} picked up${where}`;
		case 'AgentCompleted':
			return `${agentLabel} finished${where}`;
		case 'AgentCancelled':
			return `${agentLabel} was cancelled on${where}`;
		case 'AgentExpired':
			return `${agentLabel} expired on${where}`;
		case 'ToolCalled':
			return `${agentLabel} is ${toolName ? toolActivityLabel(toolName) : 'using a tool'} on${where}${isError ? ' (failed)' : ''}`;
		case 'AgentFinalizing':
			return `${agentLabel} is finalizing on${where}`;
		case 'AgentReplied':
			return `${agentLabel} replied on${where}`;
		default:
			return `${agentLabel}: ${eventType}${where}`;
	}
}
