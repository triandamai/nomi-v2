import { m } from '$lib/paraglide/messages';
import { CREW_NAMES } from '$lib/crewNames';

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
	if (CREW_NAMES[agentType]) return CREW_NAMES[agentType];
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

/** Past-tense phrases for a reply's steps, e.g. "Searched the web". */
const STEP_LABELS: Record<string, () => string> = {
	web_search: m.step_web_search,
	read_web_page: m.step_read_web_page,
	read_attachment: m.step_read_attachment,
	delegate_to_agent: m.step_delegate,
	create_project: m.step_create_project,
	write_file: m.step_write_file,
	write_files: m.step_write_files,
	edit_file: m.step_edit_file,
	delete_file: m.step_delete_file,
	read_file: m.step_read_file,
	search_files: m.step_search_files,
	run_command: m.step_run_command,
	create_reminder: m.step_create_reminder,
	list_reminders: m.step_list_reminders,
	cancel_reminder: m.step_cancel_reminder,
	list_transactions: m.step_list_transactions,
	summarize_budget: m.step_summarize_budget,
};

/** What a step did, in the past tense; a hand-off names who it went to. */
export function stepLabel(tool: string, detail: string | null): string {
	if (tool === 'delegate_to_agent' && detail) return m.step_delegate_to({ name: agentTypeFallbackLabel(detail) });
	return STEP_LABELS[tool]?.() ?? m.step_used({ tool: humanize(tool) });
}

/** A source's site, for its chip: "kopinako.id". */
export function sourceSite(url: string): string {
	try {
		return new URL(url).hostname.replace(/^www\./, '');
	} catch {
		return url;
	}
}

/** Redacts sensitive query parameters (key, token, api_key, etc.) and credentials from error messages. */
export function sanitizeError(err: string | null | undefined): string {
	if (!err) return '';
	let out = err;
	for (const [pattern, replacement] of SECRET_PATTERNS) out = out.replace(pattern, replacement);
	return out;
}

/** Secrets that can turn up in error text (the same patterns the API strips, see nomi_llm::redact):
 * keys and tokens in URLs, headers and JSON, passwords in connection strings, and well-known
 * key formats. */
const SECRET_PATTERNS: [RegExp, string][] = [
	[/([?&;\s,]?(?:api[_-]?key|key|token|access[_-]?token|refresh[_-]?token|id[_-]?token|auth|secret|client[_-]?secret|password|passwd|pwd|signature|sig)=)[^&\s)\]'"<>,;]+/gi, '$1[REDACTED]'],
	[/(["']?(?:api[_-]?key|apikey|x-api-key|x-goog-api-key|token|access[_-]?token|refresh[_-]?token|secret|client[_-]?secret|password|authorization)["']?\s*[:=]\s*["'])[^"']+/gi, '$1[REDACTED]'],
	[/(\bbearer\s+)[A-Za-z0-9._~+/=-]{12,}/gi, '$1[REDACTED]'],
	[/(\b[a-zA-Z][a-zA-Z0-9+.-]*:\/\/[^\s:/@]+:)[^\s@/]+@/g, '$1[REDACTED]@'],
	[/\bsk-ant-[A-Za-z0-9_-]{16,}/g, '[REDACTED]'],
	[/\bsk-(?:proj-|or-v1-)?[A-Za-z0-9_-]{20,}/g, '[REDACTED]'],
	[/\bAIza[0-9A-Za-z_-]{30,}/g, '[REDACTED]'],
	[/\bAQ\.[0-9A-Za-z_-]{20,}/g, '[REDACTED]'],
	[/\b(?:ghp|gho|ghu|ghs|ghr)_[A-Za-z0-9]{30,}/g, '[REDACTED]'],
	[/\bgithub_pat_[A-Za-z0-9_]{30,}/g, '[REDACTED]'],
	[/\bxox[abposr]-[A-Za-z0-9-]{10,}/g, '[REDACTED]'],
	[/\b(?:AKIA|ASIA)[0-9A-Z]{16}\b/g, '[REDACTED]'],
	[/\b(?:ship_int|whsec|nsk|tvly|sk_live|sk_test|rk_live|pk_live)_[A-Za-z0-9-]{12,}/g, '[REDACTED]'],
	[/\beyJ[A-Za-z0-9_-]{8,}\.eyJ[A-Za-z0-9_-]{8,}\.[A-Za-z0-9_-]{8,}/g, '[REDACTED]'],
];

/** "Couldn't reach the model. Trying again in 8s (attempt 3 of 10)" from a `retrying` phase. */
export function retryActivity(detail: string | null, now: number = Date.now()): string {
	let attempt = 2;
	let max = 10;
	let seconds = 0;
	try {
		const parsed = JSON.parse(detail ?? '{}') as { attempt?: number; max?: number; retry_at?: string };
		attempt = parsed.attempt ?? attempt;
		max = parsed.max ?? max;
		if (parsed.retry_at) seconds = Math.max(0, Math.ceil((new Date(parsed.retry_at).getTime() - now) / 1000));
	} catch {
		// An older or odd detail: the generic wording below still reads right.
	}
	return seconds > 0 ? m.phase_retrying_in({ seconds, attempt, max }) : m.phase_retrying_now({ attempt, max });
}

/** "running `npm install` (12s)" from a `running_command` phase. */
export function commandActivity(detail: string | null, now: number = Date.now()): string {
	let command = '';
	let seconds = 0;
	try {
		const parsed = JSON.parse(detail ?? '{}') as { command?: string; started_at?: string };
		command = parsed.command ?? '';
		if (parsed.started_at) seconds = Math.max(0, Math.round((now - new Date(parsed.started_at).getTime()) / 1000));
	} catch {
		command = detail ?? '';
	}
	const shown = command.length > 60 ? `${command.slice(0, 57)}…` : command;
	return seconds >= 3 ? m.phase_running_command_for({ command: shown, seconds }) : m.phase_running_command({ command: shown });
}
