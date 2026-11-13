import { createLiveLink, type LinkSocket } from '$lib/live/link';

// The chat's own live feed (MQTT `session/{id}/stream`, relayed at /chat/:id/ws): the reply as
// it's written, turns ending, messages arriving, changing or leaving, the chat renamed or
// deleted, and what the crew is doing. Envelopes are nomi_realtime::StreamEnvelope. Missed events
// aren't replayed, so whoever listens reloads on every (re)connect (`opened`).

export type ChatEvent =
	| { kind: 'Delta'; turn_job_id: string; event: unknown }
	| { kind: 'TurnCompleted'; turn_job_id: string; message_id: string }
	| { kind: 'TurnFailed'; turn_job_id: string; error: string }
	| { kind: 'MessageCreated'; message_id: string }
	| { kind: 'MessageUpdated'; message_id: string }
	| { kind: 'MessagesDeleted'; message_ids: string[] }
	| { kind: 'SessionRenamed'; title: string }
	| { kind: 'SessionDeleted' }
	| { kind: 'ChatSettled'; outcome: string; waiting: string | null; agent: string | null }
	| { kind: 'AgentPhaseChanged'; agent_session_id: string; phase: string; detail: string | null; agent_display_name?: string; subject?: string }
	| { kind: 'AgentSessionStarted'; agent_session_id: string; agent_type: string; agent_display_name: string }
	| { kind: 'AgentSessionEnded'; agent_session_id: string; reason: string }
	| { kind: 'AgentDelegationUpdated'; delegation_id: string }
	| { kind: 'PersonalityChanged' };

type Raw = Record<string, unknown>;
const text = (value: unknown): value is string => typeof value === 'string';
const optionalText = (value: unknown) => value === undefined || value === null || typeof value === 'string';

/** Each kind's own fields, checked: a malformed envelope is dropped, never half-read. */
const SHAPES: Record<ChatEvent['kind'], (raw: Raw) => boolean> = {
	Delta: (raw) => text(raw.turn_job_id) && 'event' in raw,
	TurnCompleted: (raw) => text(raw.turn_job_id) && text(raw.message_id),
	TurnFailed: (raw) => text(raw.turn_job_id) && text(raw.error),
	MessageCreated: (raw) => text(raw.message_id),
	MessageUpdated: (raw) => text(raw.message_id),
	MessagesDeleted: (raw) => Array.isArray(raw.message_ids) && raw.message_ids.every(text),
	SessionRenamed: (raw) => text(raw.title),
	SessionDeleted: () => true,
	ChatSettled: (raw) => text(raw.outcome) && optionalText(raw.waiting) && optionalText(raw.agent),
	AgentPhaseChanged: (raw) => text(raw.phase) && optionalText(raw.detail) && optionalText(raw.agent_display_name) && optionalText(raw.subject),
	AgentSessionStarted: (raw) => text(raw.agent_display_name),
	AgentSessionEnded: () => true,
	AgentDelegationUpdated: () => true,
	PersonalityChanged: () => true,
};

/** One envelope from the feed, or null when it isn't one this build knows. */
export function parseChatEvent(data: unknown): ChatEvent | null {
	let raw: unknown;
	try {
		raw = JSON.parse(String(data));
	} catch {
		return null;
	}
	if (typeof raw !== 'object' || raw === null || Array.isArray(raw)) return null;
	const kind = (raw as Raw).kind;
	if (!text(kind) || !(kind in SHAPES)) return null;
	return SHAPES[kind as ChatEvent['kind']](raw as Raw) ? (raw as ChatEvent) : null;
}

export interface ChatHandlers {
	/** The feed opened; `resumed` after a drop. Either way, reload: nothing missed is replayed. */
	opened(resumed: boolean): void;
	event(event: ChatEvent): void;
	/** The server ended it for good: 4401 the sign-in is over, 4403/4404 the chat is gone. */
	lost(code: number): void;
}

type Options = { sessionId: string; handlers: ChatHandlers; openSocket?: (url: string) => LinkSocket };

/** Follows one chat's feed (reconnecting when it drops); returns the function that closes it. */
export function connectChat({ sessionId, handlers, openSocket = (url) => new WebSocket(url) }: Options): () => void {
	const url = `/chat/${encodeURIComponent(sessionId)}/ws`;
	const link = createLiveLink(
		{
			opened: (resumed) => handlers.opened(resumed),
			message(data) {
				const event = parseChatEvent(data);
				if (event) handlers.event(event);
			},
			lost: (code) => handlers.lost(code),
		},
		() => openSocket(url),
	);
	return () => link.close();
}
