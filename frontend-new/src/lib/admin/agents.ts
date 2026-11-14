import { m } from '$lib/paraglide/messages';
import { CREW } from '$lib/ds/agents/crew';
import { MASCOT } from '$lib/adapters/shell';
import type { AgentEvent, AgentFrame, AgentsResponse, RunningAgent } from './api';

// Live agents: everyone's agents at work (GET /api/admin/agents) kept current by the admin stream
// (/admin/agents/ws). The stream sends no snapshot and replays nothing, so the page loads the list,
// then applies each frame, and reloads after a reconnect.

export type LiveRow = RunningAgent & { person: string };
export type FeedItem = AgentEvent;
export type LiveState = { rows: LiveRow[]; feed: FeedItem[] };

const FEED_LIMIT = 200;

export function liveRows(response: AgentsResponse): LiveRow[] {
	return response.users.flatMap((user) => user.agents.map((agent) => ({ ...agent, person: user.label })));
}

/**
 * What to call an agent. Built-in ones go by their crew name (the list says "Money", the stream
 * "Dana": the crew name is what people know); custom ones by the name they were given.
 */
export function agentName(type: string | null, displayName: string | null): string {
	const crew = type ? MASCOT[type] : undefined;
	if (crew) return CREW.find((member) => member.id === crew)?.name ?? displayName ?? crew;
	return displayName?.trim() || m.admin_agent_unknown();
}

export const shortId = (id: string) => id.slice(0, 8);

const KINDS = new Set(['AgentSessionStarted', 'AgentSessionEnded', 'AgentPhaseChanged']);

/** A frame from the stream, or null for anything else. */
export function parseFrame(data: unknown): AgentFrame | null {
	if (typeof data !== 'string') return null;
	try {
		const frame = JSON.parse(data) as { kind?: unknown };
		return frame && typeof frame.kind === 'string' && KINDS.has(frame.kind) ? (frame as AgentFrame) : null;
	} catch {
		return null;
	}
}

let made = 0;
function feedItem(row: Pick<LiveRow, 'agent_type' | 'agent_display_name'> | undefined, frame: AgentFrame, eventType: string, now: string, tool: string | null = null): FeedItem {
	return {
		id: `live-${made++}`,
		session_id: frame.session_id,
		agent_session_id: frame.agent_session_id,
		agent_type: row?.agent_type ?? null,
		agent_display_name: row?.agent_display_name ?? null,
		event_type: eventType,
		created_at: now,
		tool_name: tool,
		is_error: null,
	};
}

/** The list and the feed after one frame. A start already listed (from the snapshot) isn't added twice. */
export function applyFrame(current: LiveState, frame: AgentFrame, now: string): LiveState {
	const add = (item: FeedItem | null, rows = current.rows): LiveState => ({ rows, feed: item ? [item, ...current.feed].slice(0, FEED_LIMIT) : current.feed });
	const known = current.rows.find((row) => row.agent_session_id === frame.agent_session_id);
	switch (frame.kind) {
		case 'AgentSessionStarted': {
			const item = feedItem(frame, frame, 'AgentSpawned', now);
			if (known) return add(item);
			const row: LiveRow = {
				agent_session_id: frame.agent_session_id,
				session_id: frame.session_id,
				agent_type: frame.agent_type,
				agent_display_name: frame.agent_display_name,
				channel: frame.channel,
				started_at: now,
				last_activity_at: now,
				current_phase: 'waiting',
				current_phase_detail: null,
				person: frame.sender_label,
			};
			return add(item, [...current.rows, row]);
		}
		case 'AgentSessionEnded': {
			const type = frame.reason === 'cancelled' ? 'AgentCancelled' : frame.reason === 'expired' ? 'AgentExpired' : 'AgentCompleted';
			return add(
				feedItem(known, frame, type, now),
				current.rows.filter((row) => row.agent_session_id !== frame.agent_session_id),
			);
		}
		case 'AgentPhaseChanged': {
			const rows = current.rows.map((row) => (row.agent_session_id === frame.agent_session_id ? { ...row, current_phase: frame.phase, current_phase_detail: frame.detail, last_activity_at: now } : row));
			if (frame.phase === 'calling_tool' && frame.detail) return add(feedItem(known, frame, 'ToolCalled', now, frame.detail), rows);
			if (frame.phase === 'writing_reply') return add(feedItem(known, frame, 'AgentFinalizing', now), rows);
			return add(null, rows);
		}
	}
}

function detailJson(detail: string | null): Record<string, unknown> | null {
	try {
		const parsed = detail ? JSON.parse(detail) : null;
		return parsed && typeof parsed === 'object' ? (parsed as Record<string, unknown>) : null;
	} catch {
		return null;
	}
}

/** The step an agent is on, in words ("using read_file", "retrying, 2 of 5"). */
export function phaseLabel(phase: string, detail: string | null): string {
	switch (phase) {
		case 'waiting':
			return m.admin_phase_waiting();
		case 'thinking':
			return m.admin_phase_thinking();
		case 'calling_tool':
			return detail ? m.admin_phase_tool({ tool: detail }) : m.admin_phase_tool_unknown();
		case 'writing_reply':
			return m.admin_phase_writing();
		case 'retrying': {
			const retry = detailJson(detail);
			return typeof retry?.attempt === 'number' && typeof retry.max === 'number' ? m.admin_phase_retrying_count({ attempt: retry.attempt, max: retry.max }) : m.admin_phase_retrying();
		}
		case 'running_command':
			return m.admin_phase_command();
		default:
			return phase.replace(/_/g, ' ');
	}
}

/** The command an agent is running, when it is running one (shown beside the step, in mono). */
export function runningCommand(phase: string, detail: string | null): string | null {
	if (phase !== 'running_command') return null;
	const command = detailJson(detail)?.command;
	return typeof command === 'string' ? command : null;
}

/** One event of the feed, as a sentence. */
export function eventSentence(event: AgentEvent): string {
	const agent = agentName(event.agent_type, event.agent_display_name);
	const chat = event.session_id ? shortId(event.session_id) : null;
	const tool = event.tool_name ?? '';
	switch (event.event_type) {
		case 'AgentSpawned':
			return chat ? m.admin_event_spawned({ agent, chat }) : m.admin_event_spawned_bare({ agent });
		case 'AgentCompleted':
			return chat ? m.admin_event_completed({ agent, chat }) : m.admin_event_completed_bare({ agent });
		case 'AgentCancelled':
			return chat ? m.admin_event_cancelled({ agent, chat }) : m.admin_event_cancelled_bare({ agent });
		case 'AgentExpired':
			return chat ? m.admin_event_expired({ agent, chat }) : m.admin_event_expired_bare({ agent });
		case 'ToolCalled':
			if (event.is_error) return chat ? m.admin_event_tool_failed({ agent, tool, chat }) : m.admin_event_tool_failed_bare({ agent, tool });
			return chat ? m.admin_event_tool({ agent, tool, chat }) : m.admin_event_tool_bare({ agent, tool });
		case 'AgentReplied':
			return chat ? m.admin_event_replied({ agent, chat }) : m.admin_event_replied_bare({ agent });
		case 'AgentFinalizing':
			return chat ? m.admin_event_finalizing({ agent, chat }) : m.admin_event_finalizing_bare({ agent });
		default:
			return chat ? m.admin_event_other({ agent, event: event.event_type, chat }) : m.admin_event_other_bare({ agent, event: event.event_type });
	}
}

/** The crew member whose mascot an agent wears, if it is a built-in one. */
export const mascotFor = (type: string | null) => (type ? MASCOT[type] : undefined);
